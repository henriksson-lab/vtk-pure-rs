use crate::data::PolyData;

/// Extract the largest connected component from a PolyData.
///
/// Convenience wrapper around `extract_component` with `component_id = 0`.
pub fn extract_largest(input: &PolyData) -> PolyData {
    crate::filters::core::extract_component::extract_component(input, 0)
}

/// Extract the N largest connected components.
pub fn extract_n_largest(input: &PolyData, n: usize) -> Vec<PolyData> {
    let num = crate::filters::core::extract_component::num_components(input);
    let take = n.min(num);
    (0..take)
        .map(|i| crate::filters::core::extract_component::extract_component(input, i))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_of_two() {
        let mut pd = PolyData::new();
        // Small component: 1 triangle
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.0, 0.1, 0.0]);
        // Large component: 2 triangles (more points)
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([6.0, 1.0, 0.0]);
        pd.points.push([5.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[3, 5, 6]);

        let result = extract_largest(&pd);
        assert_eq!(result.polys.num_cells(), 2); // the larger component
    }

    #[test]
    fn extract_n() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([5.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let results = extract_n_largest(&pd, 5); // ask for 5, only 2 exist
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = extract_largest(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
