use itertools::Itertools;

/// Generate all combinations of selecting `k` features from `n` features (1-indexed)
pub fn generate_combinations(n: usize, k: usize) -> impl Iterator<Item = Vec<usize>> {
    (1..=n).combinations(k)
}

/// Batch generator to avoid loading all combinations into memory
pub struct BatchGenerator {
    combinations: Vec<Vec<usize>>,
    batch_size: usize,
    current: usize,
}

impl BatchGenerator {
    pub fn new(n: usize, k: usize, batch_size: usize) -> Self {
        // Pre-generate all combinations for now
        // For very large combinations, consider streaming approach
        let combinations: Vec<Vec<usize>> = generate_combinations(n, k).collect();
        Self {
            combinations,
            batch_size,
            current: 0,
        }
    }
    
    #[allow(dead_code)]
    pub fn total_count(&self) -> usize {
        self.combinations.len()
    }
    
    pub fn next_batch(&mut self) -> Option<Vec<Vec<usize>>> {
        if self.current >= self.combinations.len() {
            return None;
        }
        
        let end = (self.current + self.batch_size).min(self.combinations.len());
        let batch = self.combinations[self.current..end].to_vec();
        self.current = end;
        
        Some(batch)
    }
}

/// Calculate combination count C(n, k)
pub fn combination_count(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result: usize = 1;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_combination_count() {
        assert_eq!(combination_count(43, 4), 123_410);
        assert_eq!(combination_count(43, 5), 962_598);
        assert_eq!(combination_count(5, 2), 10);
    }
    
    #[test]
    fn test_generate_combinations() {
        let combos: Vec<_> = generate_combinations(5, 2).collect();
        assert_eq!(combos.len(), 10);
        assert_eq!(combos[0], vec![1, 2]);
        assert_eq!(combos[9], vec![4, 5]);
    }
}
