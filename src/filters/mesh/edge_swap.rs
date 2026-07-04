use crate::data::{CellArray, PolyData};
use std::collections::HashMap;

/// Optimize mesh quality by swapping interior edges where the swap improves
/// the minimum angle of the two affected triangles.
///
/// Only operates on triangle meshes. Each interior edge (shared by exactly two
/// triangles) is considered for swapping. If the swap produces a larger minimum
/// angle across the two resulting triangles, the swap is performed.
///
/// Performs a single pass over all interior edges.
pub fn optimize_by_edge_swap(input: &PolyData) -> PolyData {
    // Collect all triangles as index triples
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    let mut poly_cells: Vec<Vec<i64>> = Vec::new();
    let mut triangle_for_poly: Vec<Option<usize>> = Vec::new();
    let n_points = input.points.len();

    for cell in input.polys.iter() {
        poly_cells.push(cell.to_vec());
        if cell.len() == 3 {
            let tri = [cell[0], cell[1], cell[2]];
            if let (Some(a), Some(b), Some(c)) = (
                valid_point_index(tri[0], n_points),
                valid_point_index(tri[1], n_points),
                valid_point_index(tri[2], n_points),
            ) {
                if a != b && b != c && a != c {
                    triangle_for_poly.push(Some(triangles.len()));
                    triangles.push([a, b, c]);
                } else {
                    triangle_for_poly.push(None);
                }
            } else {
                triangle_for_poly.push(None);
            }
        } else {
            triangle_for_poly.push(None);
        }
    }

    if triangles.is_empty() {
        return input.clone();
    }

    // Build edge -> [tri_index] map
    // An edge is stored as (min_vertex, max_vertex)
    let mut edge_tris: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (ti, tri) in triangles.iter().enumerate() {
        for k in 0..3 {
            let a: usize = tri[k];
            let b: usize = tri[(k + 1) % 3];
            let edge = if a < b { (a, b) } else { (b, a) };
            edge_tris.entry(edge).or_default().push(ti);
        }
    }

    // Process each interior edge
    let mut swapped: Vec<bool> = vec![false; triangles.len()];

    for (&(ea, eb), tris) in &edge_tris {
        if tris.len() != 2 {
            continue;
        }
        let ti0: usize = tris[0];
        let ti1: usize = tris[1];

        // Skip if either triangle was already modified this pass
        if swapped[ti0] || swapped[ti1] {
            continue;
        }

        let t0 = triangles[ti0];
        let t1 = triangles[ti1];

        // Find the opposite vertices (not on the shared edge)
        let opp0: usize = find_opposite(&t0, ea, eb);
        let opp1: usize = find_opposite(&t1, ea, eb);
        if opp0 == opp1 {
            continue;
        }

        // Current minimum angle across both triangles
        let current_min: f64 = min_angle_of_tri(input, &t0).min(min_angle_of_tri(input, &t1));

        // Proposed swap: replace edge (ea, eb) with edge (opp0, opp1)
        // New triangles: (opp0, opp1, ea) and (opp0, opp1, eb)
        let new_t0: [usize; 3] = orient_like(input, [opp0, opp1, ea], &t0);
        let new_t1: [usize; 3] = orient_like(input, [opp0, eb, opp1], &t1);

        // Check that the new triangles are valid (non-degenerate, correct winding)
        if !has_positive_area(input, &new_t0) || !has_positive_area(input, &new_t1) {
            continue;
        }
        let new_min: f64 = min_angle_of_tri(input, &new_t0).min(min_angle_of_tri(input, &new_t1));

        if new_min > current_min + 1e-10 {
            triangles[ti0] = new_t0;
            triangles[ti1] = new_t1;
            swapped[ti0] = true;
            swapped[ti1] = true;
        }
    }

    let mut out_polys = CellArray::new();
    for (poly_id, cell) in poly_cells.iter().enumerate() {
        if let Some(tri_id) = triangle_for_poly[poly_id] {
            let tri = triangles[tri_id];
            out_polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
        } else {
            out_polys.push_cell(cell);
        }
    }

    let mut output = input.clone();
    output.polys = out_polys;
    output
}

/// Find the vertex in the triangle that is NOT ea or eb.
fn find_opposite(tri: &[usize; 3], ea: usize, eb: usize) -> usize {
    for &v in tri {
        if v != ea && v != eb {
            return v;
        }
    }
    tri[0] // fallback (should not happen with valid input)
}

/// Compute the minimum interior angle of a triangle (in radians).
fn min_angle_of_tri(pd: &PolyData, tri: &[usize; 3]) -> f64 {
    let p0 = pd.points.get(tri[0]);
    let p1 = pd.points.get(tri[1]);
    let p2 = pd.points.get(tri[2]);

    let a01: f64 = angle_at_vertex(&p0, &p1, &p2);
    let a12: f64 = angle_at_vertex(&p1, &p2, &p0);
    let a20: f64 = angle_at_vertex(&p2, &p0, &p1);

    a01.min(a12).min(a20)
}

/// Angle at vertex `v` in triangle (v, a, b).
fn angle_at_vertex(v: &[f64; 3], a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let va: [f64; 3] = [a[0] - v[0], a[1] - v[1], a[2] - v[2]];
    let vb: [f64; 3] = [b[0] - v[0], b[1] - v[1], b[2] - v[2]];

    let dot: f64 = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    let la: f64 = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
    let lb: f64 = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();

    let denom: f64 = la * lb;
    if denom < 1e-30 {
        return 0.0;
    }

    let cos_val: f64 = (dot / denom).clamp(-1.0, 1.0);
    cos_val.acos()
}

fn orient_like(pd: &PolyData, mut tri: [usize; 3], reference: &[usize; 3]) -> [usize; 3] {
    let n = triangle_normal(pd, &tri);
    let ref_n = triangle_normal(pd, reference);
    let dot = n[0] * ref_n[0] + n[1] * ref_n[1] + n[2] * ref_n[2];
    if dot < 0.0 {
        tri.swap(1, 2);
    }
    tri
}

fn triangle_normal(pd: &PolyData, tri: &[usize; 3]) -> [f64; 3] {
    let p0 = pd.points.get(tri[0]);
    let p1 = pd.points.get(tri[1]);
    let p2 = pd.points.get(tri[2]);
    let a = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let b = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn has_positive_area(pd: &PolyData, tri: &[usize; 3]) -> bool {
    let n = triangle_normal(pd, tri);
    n[0] * n[0] + n[1] * n[1] + n[2] * n[2] > 1e-30
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_improves_thin_triangles() {
        // Create a "bowtie" of two triangles sharing edge (1,2) where swapping helps.
        // Triangle 0: (0, 1, 2) and Triangle 1: (1, 2, 3)
        // Points arranged so the shared edge is long and the opposite vertices
        // are close, making thin triangles that benefit from a swap.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 2.0, 0.0], // 0: top
                [0.0, 0.0, 0.0], // 1: bottom-left
                [4.0, 0.0, 0.0], // 2: bottom-right (shared edge with 1)
                [4.0, 2.0, 0.0], // 3: top-right
            ],
            vec![[0, 1, 2], [2, 3, 0]],
        );
        let result = optimize_by_edge_swap(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn equilateral_triangles_unchanged() {
        // Two equilateral-ish triangles sharing an edge - no swap should improve things
        let s: f64 = 3.0_f64.sqrt() / 2.0;
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, s, 0.0],
                [0.5, -s, 0.0],
            ],
            vec![[0, 1, 2], [1, 0, 3]],
        );
        let result = optimize_by_edge_swap(&pd);
        assert_eq!(result.polys.num_cells(), 2);
        // Points should be unchanged
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::default();
        let result = optimize_by_edge_swap(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn invalid_and_degenerate_triangles_are_preserved() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 99]);
        pd.polys.push_cell(&[0, 1, 1]);

        let result = optimize_by_edge_swap(&pd);
        assert_eq!(result.polys.num_cells(), 3);
        assert_eq!(result.polys.cell(1), &[0, 1, 99]);
        assert_eq!(result.polys.cell(2), &[0, 1, 1]);
    }
}
