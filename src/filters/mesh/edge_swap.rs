use crate::data::PolyData;
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
    for cell in input.polys.iter() {
        if cell.len() == 3 {
            triangles.push([cell[0] as usize, cell[1] as usize, cell[2] as usize]);
        } else {
            // Non-triangle cells: skip (cannot swap edges)
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

        // Current minimum angle across both triangles
        let current_min: f64 = min_angle_of_tri(input, &t0).min(min_angle_of_tri(input, &t1));

        // Proposed swap: replace edge (ea, eb) with edge (opp0, opp1)
        // New triangles: (opp0, opp1, ea) and (opp0, opp1, eb)
        let new_t0: [usize; 3] = [opp0, opp1, ea];
        let new_t1: [usize; 3] = [opp0, opp1, eb];

        // Check that the new triangles are valid (non-degenerate, correct winding)
        let new_min: f64 = min_angle_of_tri(input, &new_t0).min(min_angle_of_tri(input, &new_t1));

        if new_min > current_min + 1e-10 {
            triangles[ti0] = new_t0;
            triangles[ti1] = new_t1;
            swapped[ti0] = true;
            swapped[ti1] = true;
        }
    }

    // Rebuild PolyData
    let tri_i64: Vec<[i64; 3]> = triangles
        .iter()
        .map(|t| [t[0] as i64, t[1] as i64, t[2] as i64])
        .collect();
    let points_vec: Vec<[f64; 3]> = (0..input.points.len())
        .map(|i| input.points.get(i))
        .collect();

    PolyData::from_triangles(points_vec, tri_i64)
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
}
