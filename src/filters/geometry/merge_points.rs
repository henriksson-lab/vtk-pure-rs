use crate::data::{CellArray, Points, PolyData};

/// Merge coincident points in a PolyData within a given tolerance.
///
/// Unlike `clean`, this does a simple tolerance-based merge without
/// spatial hashing, suitable for exact or near-exact duplicates.
/// Returns a new PolyData with merged points and remapped cells.
pub fn merge_points(input: &PolyData, tolerance: f64) -> PolyData {
    let n = input.points.len();
    let tol2 = tolerance * tolerance;

    let mut out_points = Points::<f64>::new();
    let mut remap = vec![0i64; n];

    for (i, rm) in remap.iter_mut().enumerate() {
        let pi = input.points.get(i);

        let mut found = false;
        for j in 0..out_points.len() {
            let pj = out_points.get(j);
            let d2 = (pi[0] - pj[0]) * (pi[0] - pj[0])
                + (pi[1] - pj[1]) * (pi[1] - pj[1])
                + (pi[2] - pj[2]) * (pi[2] - pj[2]);
            if d2 <= tol2 {
                *rm = j as i64;
                found = true;
                break;
            }
        }

        if !found {
            *rm = out_points.len() as i64;
            out_points.push(pi);
        }
    }

    let remap_cell = |cells: &CellArray| -> CellArray {
        let mut out = CellArray::new();
        for cell in cells.iter() {
            let remapped: Vec<i64> = cell.iter().map(|&id| remap[id as usize]).collect();
            out.push_cell(&remapped);
        }
        out
    };

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = remap_cell(&input.polys);
    pd.lines = remap_cell(&input.lines);
    pd.verts = remap_cell(&input.verts);
    pd.strips = remap_cell(&input.strips);
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_exact_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]); // duplicate of 0
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[2, 1, 3]); // uses duplicate

        let result = merge_points(&pd, 1e-10);
        assert_eq!(result.points.len(), 3);
        // Cell 1 should now reference point 0 instead of 2
        assert_eq!(result.polys.cell(1)[0], 0);
    }

    #[test]
    fn merge_within_tolerance() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]); // close to 0

        let result = merge_points(&pd, 0.01);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn no_merge_needed() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = merge_points(&pd, 1e-10);
        assert_eq!(result.points.len(), 3);
    }
}
