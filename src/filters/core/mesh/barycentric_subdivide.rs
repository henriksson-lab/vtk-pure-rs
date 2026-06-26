use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Barycentric subdivision of a triangle mesh.
///
/// For each triangle, inserts the centroid and three edge midpoints, then
/// connects the centroid to each edge midpoint creating 6 smaller triangles.
pub fn barycentric_subdivide(input: &PolyData) -> PolyData {
    let mut points = Points::new();
    // Copy original points
    for i in 0..input.points.len() {
        points.push(input.points.get(i));
    }

    let mut polys = CellArray::new();
    // Cache for edge midpoints: (min_id, max_id) -> new point index
    let mut edge_midpoints: HashMap<(usize, usize), i64> = HashMap::new();

    let get_or_insert_midpoint =
        |a: usize, b: usize, pts: &mut Points, cache: &mut HashMap<(usize, usize), i64>| -> i64 {
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&idx) = cache.get(&key) {
                return idx;
            }
            let pa = pts.get(a);
            let pb = pts.get(b);
            let mid: [f64; 3] = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let idx: i64 = pts.len() as i64;
            pts.push(mid);
            cache.insert(key, idx);
            idx
        };

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a: usize = cell[0] as usize;
        let b: usize = cell[1] as usize;
        let c: usize = cell[2] as usize;

        // Compute centroid
        let pa = points.get(a);
        let pb = points.get(b);
        let pc = points.get(c);
        let centroid: [f64; 3] = [
            (pa[0] + pb[0] + pc[0]) / 3.0,
            (pa[1] + pb[1] + pc[1]) / 3.0,
            (pa[2] + pb[2] + pc[2]) / 3.0,
        ];
        let ci: i64 = points.len() as i64;
        points.push(centroid);

        // Edge midpoints
        let mab: i64 = get_or_insert_midpoint(a, b, &mut points, &mut edge_midpoints);
        let mbc: i64 = get_or_insert_midpoint(b, c, &mut points, &mut edge_midpoints);
        let mca: i64 = get_or_insert_midpoint(c, a, &mut points, &mut edge_midpoints);

        let ai: i64 = a as i64;
        let bi: i64 = b as i64;
        let cci: i64 = c as i64;

        // 6 triangles: centroid connects to edge midpoints, original vertices connect
        // to adjacent edge midpoints
        polys.push_cell(&[ai, mab, ci]);
        polys.push_cell(&[mab, bi, ci]);
        polys.push_cell(&[bi, mbc, ci]);
        polys.push_cell(&[mbc, cci, ci]);
        polys.push_cell(&[cci, mca, ci]);
        polys.push_cell(&[mca, ai, ci]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_produces_six() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = barycentric_subdivide(&pd);
        assert_eq!(result.polys.num_cells(), 6);
        // 3 original + 1 centroid + 3 edge midpoints = 7
        assert_eq!(result.points.len(), 7);
    }

    #[test]
    fn two_triangles_shared_edge() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let result = barycentric_subdivide(&pd);
        assert_eq!(result.polys.num_cells(), 12);
        // 4 original + 2 centroids + 5 edge midpoints (shared edge 0-1) = 11
        assert_eq!(result.points.len(), 11);
    }

    #[test]
    fn centroid_location() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = barycentric_subdivide(&pd);
        // Centroid should be at (1, 1, 0) - it's the 4th point (index 3)
        let centroid = result.points.get(3);
        assert!((centroid[0] - 1.0).abs() < 1e-10);
        assert!((centroid[1] - 1.0).abs() < 1e-10);
        assert!((centroid[2] - 0.0).abs() < 1e-10);
    }
}
