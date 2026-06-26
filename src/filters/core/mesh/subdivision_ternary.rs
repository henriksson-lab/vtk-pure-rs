use crate::data::{CellArray, Points, PolyData};

/// Ternary subdivision: split each triangle into 9 sub-triangles.
///
/// Inserts edge third-points and a center point, creating a 3x3 grid
/// of sub-triangles. Higher refinement than 1-to-4 subdivision.
pub fn subdivide_ternary(input: &PolyData) -> PolyData {
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

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
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        let pa = input.points.get(a);
        let pb = input.points.get(b);
        let pc = input.points.get(c);

        // Edge third-points
        let ab1 = out_pts.len() as i64;
        out_pts.push(lerp(pa, pb, 1.0 / 3.0));
        let ab2 = out_pts.len() as i64;
        out_pts.push(lerp(pa, pb, 2.0 / 3.0));
        let bc1 = out_pts.len() as i64;
        out_pts.push(lerp(pb, pc, 1.0 / 3.0));
        let bc2 = out_pts.len() as i64;
        out_pts.push(lerp(pb, pc, 2.0 / 3.0));
        let ca1 = out_pts.len() as i64;
        out_pts.push(lerp(pc, pa, 1.0 / 3.0));
        let ca2 = out_pts.len() as i64;
        out_pts.push(lerp(pc, pa, 2.0 / 3.0));

        // Center point
        let center = out_pts.len() as i64;
        out_pts.push([
            (pa[0] + pb[0] + pc[0]) / 3.0,
            (pa[1] + pb[1] + pc[1]) / 3.0,
            (pa[2] + pb[2] + pc[2]) / 3.0,
        ]);

        let ai = a as i64;
        let bi = b as i64;
        let ci = c as i64;

        // 9 sub-triangles
        // Corner triangles
        out_polys.push_cell(&[ai, ab1, ca2]);
        out_polys.push_cell(&[bi, bc1, ab2]);
        out_polys.push_cell(&[ci, ca1, bc2]);
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
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(subdivide_ternary(&pd).polys.num_cells(), 0);
    }
}
