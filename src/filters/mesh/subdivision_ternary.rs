use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Ternary subdivision: split each triangle into 9 sub-triangles.
///
/// Inserts edge third-points and a center point, creating a 3x3 grid
/// of sub-triangles. Higher refinement than 1-to-4 subdivision.
pub fn subdivide_ternary(input: &PolyData) -> PolyData {
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut third_cache: HashMap<(i64, i64), (i64, i64)> = HashMap::new();

    // Copy original points
    let n = input.points.len();
    for i in 0..n {
        out_pts.push(input.points.get(i));
    }

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            out_polys.push_cell(cell);
            continue;
        }
        if !valid_cell(cell, input.points.len()) {
            out_polys.push_cell(cell);
            continue;
        }
        let a = cell[0];
        let b = cell[1];
        let c = cell[2];
        let pa = input.points.get(a as usize);
        let pb = input.points.get(b as usize);
        let pc = input.points.get(c as usize);

        // Edge third-points
        let (ab1, ab2) = get_thirds(a, b, &mut out_pts, &mut third_cache);
        let (bc1, bc2) = get_thirds(b, c, &mut out_pts, &mut third_cache);
        let (ca1, ca2) = get_thirds(c, a, &mut out_pts, &mut third_cache);

        // Center point
        let center = out_pts.len() as i64;
        out_pts.push([
            (pa[0] + pb[0] + pc[0]) / 3.0,
            (pa[1] + pb[1] + pc[1]) / 3.0,
            (pa[2] + pb[2] + pc[2]) / 3.0,
        ]);

        // 9 sub-triangles
        // Corner triangles
        out_polys.push_cell(&[a, ab1, ca2]);
        out_polys.push_cell(&[b, bc1, ab2]);
        out_polys.push_cell(&[c, ca1, bc2]);
        // Edge-center triangles
        out_polys.push_cell(&[ab1, ab2, center]);
        out_polys.push_cell(&[bc1, bc2, center]);
        out_polys.push_cell(&[ca1, ca2, center]);
        // Bridge triangles
        out_polys.push_cell(&[ab1, center, ca2]);
        out_polys.push_cell(&[ab2, bc1, center]);
        out_polys.push_cell(&[bc2, ca1, center]);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

fn get_thirds(
    a: i64,
    b: i64,
    pts: &mut Points<f64>,
    cache: &mut HashMap<(i64, i64), (i64, i64)>,
) -> (i64, i64) {
    let key = if a < b { (a, b) } else { (b, a) };
    let (low_third, high_third) = *cache.entry(key).or_insert_with(|| {
        let pa = pts.get(key.0 as usize);
        let pb = pts.get(key.1 as usize);
        let first = pts.len() as i64;
        pts.push(lerp(pa, pb, 1.0 / 3.0));
        let second = pts.len() as i64;
        pts.push(lerp(pa, pb, 2.0 / 3.0));
        (first, second)
    });

    if a <= b {
        (low_third, high_third)
    } else {
        (high_third, low_third)
    }
}

fn valid_cell(cell: &[i64], n_points: usize) -> bool {
    cell.iter()
        .all(|&id| usize::try_from(id).ok().is_some_and(|id| id < n_points))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = subdivide_ternary(&pd);
        assert_eq!(result.polys.num_cells(), 9); // 1->9
        assert_eq!(result.points.len(), 10); // 3 orig + 6 edge + 1 center
    }

    #[test]
    fn two_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = subdivide_ternary(&pd);
        assert_eq!(result.polys.num_cells(), 18); // 2*9
        assert_eq!(result.points.len(), 16); // 4 original + 2 centers + 10 third-points
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(subdivide_ternary(&pd).polys.num_cells(), 0);
    }

    #[test]
    fn invalid_triangle_is_passed_through() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 99]);

        let result = subdivide_ternary(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }
}
