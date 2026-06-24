use crate::data::{CellArray, KdTree, Points, PolyData};

/// Merge points from multiple PolyData inputs, removing duplicates within tolerance.
///
/// Takes a slice of PolyData and merges them into one, using a k-d tree
/// to find and merge coincident points. Cell connectivity is updated.
/// Point data arrays from the first input are preserved where possible.
pub fn point_merge(inputs: &[&PolyData], tolerance: f64) -> PolyData {
    if inputs.is_empty() {
        return PolyData::new();
    }
    if inputs.len() == 1 {
        return inputs[0].clone();
    }

    // Collect all points
    let mut all_pts: Vec<[f64; 3]> = Vec::new();
    let mut source_info: Vec<(usize, usize)> = Vec::new(); // (input_idx, point_idx)

    for (si, pd) in inputs.iter().enumerate() {
        for i in 0..pd.points.len() {
            all_pts.push(pd.points.get(i));
            source_info.push((si, i));
        }
    }

    // Build k-d tree and merge
    let tree = KdTree::build(&all_pts);
    let tol2 = tolerance * tolerance;

    let mut out_points = Points::<f64>::new();
    let mut remap = vec![0i64; all_pts.len()];
    let mut merged = vec![false; all_pts.len()];

    for i in 0..all_pts.len() {
        if merged[i] {
            continue;
        }

        let idx = out_points.len() as i64;
        out_points.push(all_pts[i]);
        remap[i] = idx;

        // Find nearby points and merge them
        let nearby = tree.find_within_radius(all_pts[i], tolerance);
        for &(j, d2) in &nearby {
            if j != i && !merged[j] && d2 <= tol2 {
                remap[j] = idx;
                merged[j] = true;
            }
        }
    }

    // Build offset table for point remapping per input
    let mut offsets = vec![0usize; inputs.len()];
    let mut off = 0;
    for (si, pd) in inputs.iter().enumerate() {
        offsets[si] = off;
        off += pd.points.len();
    }

    // Merge cells
    let mut out_polys = CellArray::new();
    let mut out_lines = CellArray::new();

    for (si, pd) in inputs.iter().enumerate() {
        for cell in pd.polys.iter() {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| remap[offsets[si] + id as usize])
                .collect();
            out_polys.push_cell(&mapped);
        }
        for cell in pd.lines.iter() {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| remap[offsets[si] + id as usize])
                .collect();
            out_lines.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_two_meshes() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([0.5, 1.0, 0.0]);
        a.polys.push_cell(&[0, 1, 2]);

        let mut b = PolyData::new();
        b.points.push([1.0, 0.0, 0.0]); // same as a[1]
        b.points.push([2.0, 0.0, 0.0]);
        b.points.push([1.5, 1.0, 0.0]);
        b.polys.push_cell(&[0, 1, 2]);

        let result = point_merge(&[&a, &b], 0.01);
        assert_eq!(result.points.len(), 5); // 6 - 1 merged
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn no_overlap() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([10.0, 10.0, 10.0]);
        b.points.push([11.0, 10.0, 10.0]);

        let result = point_merge(&[&a, &b], 0.01);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn single_input() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        let result = point_merge(&[&a], 0.01);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn empty() {
        let result = point_merge(&[], 0.01);
        assert_eq!(result.points.len(), 0);
    }
}
