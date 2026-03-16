// gradient_boosting.rs
use linfa::prelude::*;
use linfa_trees::DecisionTree;
use ndarray::{Array1, Array2, Axis};
use rand::SeedableRng;
use rand::rngs::StdRng;

/// Gradient Boosting Classifier for multi-class classification
/// Implements simplified sklearn-like behavior
pub struct GradientBoosting {
    trees: Vec<Vec<DecisionTree<f64, usize>>>,
    n_classes: usize,
    learning_rate: f64,
    n_estimators: usize,
    max_depth: usize,
    seed: u64,
}

impl GradientBoosting {
    /// Create a new GradientBoostingClassifier
    pub fn new(n_estimators: usize, learning_rate: f64, max_depth: usize, seed: u64) -> Self {
        Self {
            trees: Vec::with_capacity(n_estimators),
            n_classes: 0,
            learning_rate,
            n_estimators,
            max_depth,
            seed,
        }
    }

    /// Set random seed for reproducibility
    pub fn with_random_state(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Get unique classes from labels
    fn get_unique_classes(&self, y: &Array1<usize>) -> Vec<usize> {
        let mut classes: Vec<usize> = y.iter().copied().collect();
        classes.sort_unstable();
        classes.dedup();
        classes
    }        
    /// Fit the Gradient Boosting model for multi-class

    pub fn fit(mut self, x: &Array2<f64>, y: &Array1<usize>) -> anyhow::Result<Self> {
        let n_samples = x.nrows();
        let unique_classes = self.get_unique_classes(y);
        self.n_classes = unique_classes.len();
        
        // Initialize predictions for each class
        let mut class_predictions: Vec<Array1<f64>> = Vec::with_capacity(self.n_classes);
        for _ in 0..self.n_classes {
            let init_pred = Array1::from_elem(n_samples, 0.0); // Simplified initialization
            class_predictions.push(init_pred);
        }

        // Initialize RNG for reproducibility
        let mut _rng = StdRng::seed_from_u64(self.seed);

        for i in 0..self.n_estimators {
            let mut estimator_trees = Vec::with_capacity(self.n_classes);
            
            for (class_idx, &class_label) in unique_classes.iter().enumerate() {
                // Create binary problem: is this sample of current class or not?
                let binary_targets = y.mapv(|label| if label == class_label { 1.0 } else { 0.0 });
                
                // Compute pseudo-residuals for this class
                let current_class_preds = &class_predictions[class_idx];
                let residuals = &binary_targets - current_class_preds;
                
                // Fit a tree to residuals for this class
                let dataset = linfa::Dataset::new(x.clone(), residuals.mapv(|v| if v > 0.0 { 1 } else { 0 }));
                let tree = DecisionTree::params()
                    .max_depth(Some(self.max_depth))
                    .fit(&dataset)
                    .map_err(|e| anyhow::anyhow!("Tree {} for class {} training failed: {:?}", i, class_label, e))?;
            // Update predictions using the tree
                for j in 0..n_samples {
                    let row = x.row(j);
                    let pred = tree.predict(&row.insert_axis(Axis(0)));
                    let pred_val = pred[0] as f64;
                    class_predictions[class_idx][j] += self.learning_rate * pred_val;
                }
                estimator_trees.push(tree);
            }

            self.trees.push(estimator_trees);
        }

        Ok(self)
    }

    /// Predict class probabilities for multi-class
    pub fn predict_proba(&self, x: &Array2<f64>) -> Array2<f64> {
        let n_samples = x.nrows();
        let mut class_scores = Array2::<f64>::zeros((n_samples, self.n_classes));
        
        // Accumulate predictions from all trees for each class
        for estimator_trees in &self.trees {
            for (class_idx, tree) in estimator_trees.iter().enumerate() {
                for sample_idx in 0..n_samples {
                    let row = x.row(sample_idx);
                    let pred = tree.predict(&row.insert_axis(Axis(0)));
                    let pred_val = pred[0] as f64;
                    class_scores[[sample_idx, class_idx]] += self.learning_rate * pred_val;
                }
            }
        }
        
    // Apply softmax to get probabilities
        // First, subtract max for numerical stability
        let mut probs = Array2::zeros((n_samples, self.n_classes));
        for sample_idx in 0..n_samples {
            let mut max_score = f64::NEG_INFINITY;
            for class_idx in 0..self.n_classes {
                if class_scores[[sample_idx, class_idx]] > max_score {
                    max_score = class_scores[[sample_idx, class_idx]];
                }
            }
            
            let mut sum_exp = 0.0;
            let mut exp_scores = Vec::with_capacity(self.n_classes);
            
            for class_idx in 0..self.n_classes {
                let exp_score = (class_scores[[sample_idx, class_idx]] - max_score).exp();
                exp_scores.push(exp_score);
                sum_exp += exp_score;
            }
            
            for class_idx in 0..self.n_classes {
                probs[[sample_idx, class_idx]] = exp_scores[class_idx] / sum_exp;
            }
        }
        
        probs
    }

    /// Predict class labels for multi-class
    pub fn predict(&self, x: &Array2<f64>) -> Array1<usize> {
        let probas = self.predict_proba(x);
        let n_samples = x.nrows();
        let mut predictions = Array1::zeros(n_samples);
        
        for sample_idx in 0..n_samples {
            let mut max_prob = -1.0;
            let mut predicted_class = 0;
            
            for class_idx in 0..self.n_classes {
                let prob = probas[[sample_idx, class_idx]];
                if prob > max_prob {
                    max_prob = prob;
                    predicted_class = class_idx;
                }
            }
            
            predictions[sample_idx] = predicted_class;
        }
        
        predictions
    }
}