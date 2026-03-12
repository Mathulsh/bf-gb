use anyhow::{Result};
use ndarray::{Array1, Array2};
use polars::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::gradient_boosting::GradientBoosting;
use crate::redis_client::{Task, TaskResult};

pub struct Dataset {
    pub features: Array2<f64>,
    pub labels: Array1<usize>,
    pub feature_names: Vec<String>,
}

impl Dataset {
    pub fn from_csv(path: &str) -> Result<Self> {
        info!("Loading dataset from {}", path);

        let df = CsvReadOptions::default()
            .try_into_reader_with_file_path(Some(path.into()))?
            .finish()?;

        let all_columns = df.get_column_names();
        let feature_names: Vec<String> = all_columns
            .into_iter()
            .filter(|&c| c.as_str() != "label")
            .map(|c| c.to_string())
            .collect();

        let n_samples = df.height();
        let n_features = feature_names.len();

        // Extract features
        let mut features = Array2::zeros((n_samples, n_features));
        for (col_idx, col_name) in feature_names.iter().enumerate() {
            let series = df.column(col_name)?;
            for row_idx in 0..n_samples {
                let val: f64 = series.get(row_idx)?.try_extract()?;
                features[[row_idx, col_idx]] = val;
            }
        }

        // Extract labels as usize
        let label_series = df.column("label")?;
        let mut labels = Array1::zeros(n_samples);
        for row_idx in 0..n_samples {
            let val: i32 = label_series.get(row_idx)?.try_extract()?;
            labels[row_idx] = val as usize;
        }

        info!(
            "Loaded dataset: {} samples, {} features",
            n_samples, n_features
        );

        Ok(Self {
            features,
            labels,
            feature_names,
        })
    }

    /// Select specific features by 1-indexed column numbers
    pub fn select_features(&self, feature_indices: &[usize]) -> Result<Array2<f64>> {
        let zero_based: Vec<usize> = feature_indices.iter().map(|&i| i - 1).collect();

        let n_samples = self.features.nrows();
        let n_selected = zero_based.len();
        let mut selected = Array2::zeros((n_samples, n_selected));

        for (new_idx, &old_idx) in zero_based.iter().enumerate() {
            if old_idx >= self.feature_names.len() {
                anyhow::bail!("Feature index {} out of range", old_idx + 1);
            }
            selected.column_mut(new_idx).assign(&self.features.column(old_idx));
        }

        Ok(selected)
    }
}

/// Stratified K-Fold cross validation
pub struct StratifiedKFold {
    n_splits: usize,
    shuffle: bool,
}

impl StratifiedKFold {
    pub fn new(n_splits: usize, shuffle: bool) -> Self {
        Self {
            n_splits,
            shuffle,
        }
    }

    pub fn split(&self, y: &Array1<usize>, seed: u64) -> Vec<(Vec<usize>, Vec<usize>)> {
        let mut class_indices: HashMap<usize, Vec<usize>> = HashMap::new();
        for (idx, &label) in y.iter().enumerate() {
            class_indices.entry(label).or_default().push(idx);
        }

        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut fold_indices: Vec<Vec<usize>> = vec![Vec::new(); self.n_splits];

        for indices in class_indices.values_mut() {
            if self.shuffle {
                indices.shuffle(&mut rng);
            }

            for (i, &idx) in indices.iter().enumerate() {
                fold_indices[i % self.n_splits].push(idx);
            }
        }

        let mut splits = Vec::with_capacity(self.n_splits);
        for i in 0..self.n_splits {
            let test_indices = fold_indices[i].clone();
            let train_indices: Vec<usize> = fold_indices
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .flat_map(|(_, fold)| fold.clone())
                .collect();
            splits.push((train_indices, test_indices));
        }

        splits
    }
}

/// Calculate F1-macro score
fn f1_macro(predictions: &Array1<usize>, ground_truth: &Array1<usize>) -> f64 {
    let mut class_metrics: HashMap<usize, (usize, usize, usize)> = HashMap::new();

    let mut classes: Vec<usize> = ground_truth.iter().copied().collect();
    classes.sort_unstable();
    classes.dedup();

    for &class in &classes {
        class_metrics.insert(class, (0, 0, 0));
    }

    for (pred, actual) in predictions.iter().zip(ground_truth.iter()) {
        if pred == actual {
            let entry = class_metrics.get_mut(pred).unwrap();
            entry.0 += 1;
        } else {
            let fp_entry = class_metrics.get_mut(pred).unwrap();
            fp_entry.1 += 1;
            let fn_entry = class_metrics.get_mut(actual).unwrap();
            fn_entry.2 += 1;
        }
    }

    let mut f1_scores = Vec::new();
    for &class in &classes {
        let (tp, fp, fn_) = class_metrics[&class];
        let precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let recall = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };
        let f1 = if precision + recall > 0.0 {
            2.0 * precision * recall / (precision + recall)
        } else {
            0.0
        };
        f1_scores.push(f1);
    }

    f1_scores.iter().sum::<f64>() / f1_scores.len() as f64
}

pub struct Trainer {
    dataset: Dataset,
    n_estimators: usize,
    learning_rate: f64,
    max_depth: usize,
    n_folds: usize,
    random_state: u64,
}

impl Trainer {
    pub fn new(dataset: Dataset, n_estimators: usize, learning_rate: f64, max_depth: usize, n_folds: usize, random_state: u64) -> Self {
        Self {
            dataset,
            n_estimators,
            learning_rate,
            max_depth,
            n_folds,
            random_state: random_state,
        }
    }

    pub fn train_and_evaluate(&self, task: &Task) -> Result<TaskResult> {
        debug!("Training with features: {:?}", task.features);

        let x = self.dataset.select_features(&task.features)?;
        let y = &self.dataset.labels;

        let cv = StratifiedKFold::new(self.n_folds, true);
        let splits = cv.split(y, self.random_state);

        let mut f1_scores = Vec::with_capacity(self.n_folds);

        for (fold_idx, (train_idx, test_idx)) in splits.iter().enumerate() {
            debug!("Processing fold {}/{}", fold_idx + 1, self.n_folds);

            let x_train = select_rows(&x, train_idx);
            let y_train = select_elements(y, train_idx);
            let x_test = select_rows(&x, test_idx);
            let y_test = select_elements(y, test_idx);

            // Create Gradient Boosting with sklearn-like defaults
            let gb = GradientBoosting::new(
                self.n_estimators,
                self.learning_rate,
                self.max_depth
            ).with_random_state(self.random_state);

            let gb = gb.fit(&x_train, &y_train)
                .map_err(|e| anyhow::anyhow!("Training failed: {:?}", e))?;

            let predictions = gb.predict(&x_test);
            let f1 = f1_macro(&predictions, &y_test);
            f1_scores.push(f1);

            debug!("Fold {} F1-macro: {:.4}", fold_idx + 1, f1);
        }

        let mean_f1 = f1_scores.iter().sum::<f64>() / f1_scores.len() as f64;
        let mean_f1_rounded = (mean_f1 * 100.0).round() / 100.0;

        info!(
            "Features {:?} -> Mean F1-macro: {:.2}",
            task.features, mean_f1_rounded
        );

        Ok(TaskResult {
            features: task.features.clone(),
            mean_f1_macro: mean_f1_rounded,
        })
    }
}

fn select_rows(array: &Array2<f64>, indices: &[usize]) -> Array2<f64> {
    let n_cols = array.ncols();
    let mut result = Array2::zeros((indices.len(), n_cols));
    for (new_idx, &old_idx) in indices.iter().enumerate() {
        result.row_mut(new_idx).assign(&array.row(old_idx));
    }
    result
}

fn select_elements(array: &Array1<usize>, indices: &[usize]) -> Array1<usize> {
    let mut result = Array1::zeros(indices.len());
    for (new_idx, &old_idx) in indices.iter().enumerate() {
        result[new_idx] = array[old_idx];
    }
    result
}