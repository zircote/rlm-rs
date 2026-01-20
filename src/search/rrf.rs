//! Reciprocal Rank Fusion (RRF) algorithm.
//!
//! Combines multiple ranked lists into a single fused ranking.
//! Based on: Cormack, Clarke, Buettcher (2009) - "Reciprocal Rank Fusion
//! outperforms Condorcet and individual Rank Learning Methods"

use std::collections::HashMap;

/// Configuration for RRF algorithm.
#[derive(Debug, Clone, Copy)]
pub struct RrfConfig {
    /// The k parameter controls how much weight is given to lower-ranked items.
    /// Higher k values give more weight to items ranked lower in the lists.
    /// Default is 60, which is the value recommended in the original paper.
    pub k: u32,
}

impl Default for RrfConfig {
    fn default() -> Self {
        Self { k: 60 }
    }
}

impl RrfConfig {
    /// Creates a new RRF config with the specified k value.
    #[must_use]
    pub const fn new(k: u32) -> Self {
        Self { k }
    }
}

/// Performs Reciprocal Rank Fusion on multiple ranked lists.
///
/// The RRF score for each item is calculated as:
/// `score(d) = Σ 1 / (k + rank(d))`
///
/// where the sum is over all ranked lists that contain item d.
///
/// # Arguments
///
/// * `ranked_lists` - Slice of ranked lists, where each list contains item IDs
///   ordered by relevance (most relevant first).
/// * `config` - RRF configuration (k parameter).
///
/// # Returns
///
/// A vector of (`item_id`, `rrf_score`) tuples, sorted by score descending.
///
/// # Examples
///
/// ```
/// use rlm_rs::search::{reciprocal_rank_fusion, RrfConfig};
///
/// let list1 = vec![1, 2, 3, 4, 5];
/// let list2 = vec![3, 1, 5, 2, 4];
///
/// let config = RrfConfig::new(60);
/// let fused = reciprocal_rank_fusion(&[&list1, &list2], &config);
///
/// // Items appearing in both lists with good ranks will score highest
/// assert!(!fused.is_empty());
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn reciprocal_rank_fusion(ranked_lists: &[&[i64]], config: &RrfConfig) -> Vec<(i64, f64)> {
    let mut scores: HashMap<i64, f64> = HashMap::new();

    for list in ranked_lists {
        for (rank, &item_id) in list.iter().enumerate() {
            // RRF formula: 1 / (k + rank)
            // rank is 0-indexed, so we add 1 to make it 1-indexed
            let rrf_score = 1.0 / f64::from(config.k + (rank as u32) + 1);
            *scores.entry(item_id).or_insert(0.0) += rrf_score;
        }
    }

    // Sort by score descending
    let mut results: Vec<(i64, f64)> = scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results
}

/// Performs weighted RRF where each list has a weight.
///
/// Useful when you want to give more importance to one retrieval method.
///
/// # Arguments
///
/// * `ranked_lists` - Slice of (list, weight) tuples.
/// * `config` - RRF configuration.
///
/// # Returns
///
/// A vector of (`item_id`, `weighted_rrf_score`) tuples, sorted by score descending.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn weighted_rrf(ranked_lists: &[(&[i64], f64)], config: &RrfConfig) -> Vec<(i64, f64)> {
    let mut scores: HashMap<i64, f64> = HashMap::new();

    for (list, weight) in ranked_lists {
        for (rank, &item_id) in list.iter().enumerate() {
            let rrf_score = weight / f64::from(config.k + (rank as u32) + 1);
            *scores.entry(item_id).or_insert(0.0) += rrf_score;
        }
    }

    let mut results: Vec<(i64, f64)> = scores.into_iter().collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_single_list() {
        let list = vec![1, 2, 3];
        let config = RrfConfig::new(60);

        let results = reciprocal_rank_fusion(&[&list], &config);

        assert_eq!(results.len(), 3);
        // First item should have highest score
        assert_eq!(results[0].0, 1);
        assert!(results[0].1 > results[1].1);
        assert!(results[1].1 > results[2].1);
    }

    #[test]
    fn test_rrf_multiple_lists() {
        let list1 = vec![1, 2, 3];
        let list2 = vec![3, 2, 1];
        let config = RrfConfig::new(60);

        let results = reciprocal_rank_fusion(&[&list1, &list2], &config);

        assert_eq!(results.len(), 3);
        // Items 1 and 3 tie (1/61 + 1/63), item 2 is slightly lower (2/62)
        // All scores are very close, verify all items present
        let ids: std::collections::HashSet<i64> = results.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }

    #[test]
    fn test_rrf_disjoint_lists() {
        let list1 = vec![1, 2];
        let list2 = vec![3, 4];
        let config = RrfConfig::new(60);

        let results = reciprocal_rank_fusion(&[&list1, &list2], &config);

        assert_eq!(results.len(), 4);
        // Items at rank 1 should be tied
        let score1 = results.iter().find(|(id, _)| *id == 1).unwrap().1;
        let score3 = results.iter().find(|(id, _)| *id == 3).unwrap().1;
        assert!((score1 - score3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rrf_empty_lists() {
        let list1: Vec<i64> = vec![];
        let config = RrfConfig::new(60);

        let results = reciprocal_rank_fusion(&[&list1], &config);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rrf_k_parameter() {
        let list = vec![1, 2];
        let config_low_k = RrfConfig::new(1);
        let config_high_k = RrfConfig::new(100);

        let results_low = reciprocal_rank_fusion(&[&list], &config_low_k);
        let results_high = reciprocal_rank_fusion(&[&list], &config_high_k);

        // With low k, score difference between ranks is larger
        let diff_low = results_low[0].1 - results_low[1].1;
        let diff_high = results_high[0].1 - results_high[1].1;

        assert!(diff_low > diff_high);
    }

    #[test]
    fn test_weighted_rrf() {
        let list1 = vec![1, 2];
        let list2 = vec![2, 1];
        let config = RrfConfig::new(60);

        // Give double weight to list1
        let results = weighted_rrf(&[(&list1, 2.0), (&list2, 1.0)], &config);

        // Item 1 should win because it's rank 1 in the higher-weighted list
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_rrf_score_formula() {
        let list = vec![1];
        let config = RrfConfig::new(60);

        let results = reciprocal_rank_fusion(&[&list], &config);

        // Score should be 1 / (60 + 1) = 1/61
        let expected = 1.0 / 61.0;
        assert!((results[0].1 - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rrf_combined_score() {
        let list1 = vec![1];
        let list2 = vec![1];
        let config = RrfConfig::new(60);

        let results = reciprocal_rank_fusion(&[&list1, &list2], &config);

        // Score should be 2 * (1 / 61) = 2/61
        let expected = 2.0 / 61.0;
        assert!((results[0].1 - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rrf_config_default() {
        // Test Default trait implementation (line 19)
        let config = RrfConfig::default();
        assert_eq!(config.k, 60);
    }
}
