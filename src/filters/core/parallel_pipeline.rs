//! Parallel pipeline execution for independent filter branches.
//!
//! Allows running multiple filter chains concurrently using rayon,
//! then merging the results.

use crate::data::PolyData;
use rayon::prelude::*;

/// A pipeline branch: a function that transforms a PolyData.
pub type FilterFn = Box<dyn Fn(&PolyData) -> PolyData + Send + Sync>;

/// Execute multiple filter branches in parallel on the same input.
///
/// Returns a vector of results, one per branch.
pub fn parallel_branches(input: &PolyData, branches: &[FilterFn]) -> Vec<PolyData> {
    branches.par_iter().map(|f| f(input)).collect()
}

/// Execute multiple filter branches and merge results.
pub fn parallel_branches_merge(input: &PolyData, branches: &[FilterFn]) -> PolyData {
    let results = parallel_branches(input, branches);
    let refs: Vec<&PolyData> = results.iter().collect();
    if refs.is_empty() {
        return PolyData::new();
    }
    crate::filters::core::append::append(&refs)
}

/// Process multiple independent inputs in parallel.
pub fn parallel_map(
    inputs: &[PolyData],
    f: impl Fn(&PolyData) -> PolyData + Send + Sync,
) -> Vec<PolyData> {
    inputs.par_iter().map(|input| f(input)).collect()
}

/// Process and merge multiple independent inputs.
pub fn parallel_map_merge(
    inputs: &[PolyData],
    f: impl Fn(&PolyData) -> PolyData + Send + Sync,
) -> PolyData {
    let results = parallel_map(inputs, f);
    let refs: Vec<&PolyData> = results.iter().collect();
    if refs.is_empty() {
        return PolyData::new();
    }
    crate::filters::core::append::append(&refs)
}

/// Split input into N chunks, process each in parallel, then merge.
pub fn parallel_chunked(
    input: &PolyData,
    n_chunks: usize,
    f: impl Fn(&PolyData) -> PolyData + Send + Sync,
) -> PolyData {
    let chunks = crate::filters::core::piece_request::split_points_into_pieces(input, n_chunks);
    parallel_map_merge(&chunks, f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_two_branches() {
        let input = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let branches: Vec<FilterFn> = vec![Box::new(|pd| pd.clone()), Box::new(|pd| pd.clone())];
        let results = parallel_branches(&input, &branches);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn parallel_merge() {
        let input = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let branches: Vec<FilterFn> = vec![Box::new(|pd| pd.clone()), Box::new(|pd| pd.clone())];
        let merged = parallel_branches_merge(&input, &branches);
        assert_eq!(merged.points.len(), 2); // two copies merged
    }

    #[test]
    fn parallel_map_test() {
        let inputs = vec![
            PolyData::from_points(vec![[0.0, 0.0, 0.0]]),
            PolyData::from_points(vec![[1.0, 0.0, 0.0]]),
        ];
        let results = parallel_map(&inputs, |pd| pd.clone());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn chunked_processing() {
        let input =
            PolyData::from_points((0..20).map(|i| [i as f64, 0.0, 0.0]).collect::<Vec<_>>());
        let result = parallel_chunked(&input, 4, |chunk| chunk.clone());
        assert_eq!(result.points.len(), 20);
    }
}
