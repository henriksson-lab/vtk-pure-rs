use std::collections::HashMap;

use crate::data::PolyData;

const COS_EDGE_ANGLE: f64 = 0.9659258262890683; // cos(15 degrees), VTK default EdgeAngle.

/// Laplacian smoothing of a PolyData mesh.
pub fn smooth(
    input: &PolyData,
    iterations: usize,
    relaxation_factor: f64,
    fix_boundary: bool,
) -> PolyData {
    let mut output = input.clone();
    let n = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    let neighbors = build_smoothing_neighbors(input, n, fix_boundary);
    let factor = relaxation_factor;

    // Work directly on flat f64 buffer for cache efficiency
    let mut pos: Vec<f64> = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = output.points.get(i);
        pos.extend_from_slice(&p);
    }
    for _ in 0..iterations {
        for i in 0..n {
            let nbrs = &neighbors[i];
            if nbrs.is_empty() {
                continue;
            }

            let mut ax = 0.0f64;
            let mut ay = 0.0f64;
            let mut az = 0.0f64;
            for &nb in nbrs {
                unsafe {
                    ax += *pos.get_unchecked(nb * 3);
                    ay += *pos.get_unchecked(nb * 3 + 1);
                    az += *pos.get_unchecked(nb * 3 + 2);
                }
            }
            let inv = 1.0 / nbrs.len() as f64;
            ax *= inv;
            ay *= inv;
            az *= inv;

            let base = i * 3;
            pos[base] += factor * (ax - pos[base]);
            pos[base + 1] += factor * (ay - pos[base + 1]);
            pos[base + 2] += factor * (az - pos[base + 2]);
        }
    }

    // Write back
    for i in 0..n {
        output
            .points
            .set(i, [pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]]);
    }
    output
}

/// Laplacian smoothing entry point kept for compatibility with older callers.
pub fn smooth_par(
    input: &PolyData,
    iterations: usize,
    relaxation_factor: f64,
    fix_boundary: bool,
) -> PolyData {
    smooth(input, iterations, relaxation_factor, fix_boundary)
}

fn build_smoothing_neighbors(input: &PolyData, n: usize, fix_boundary: bool) -> Vec<Vec<usize>> {
    let mut fixed = vec![false; n];
    for cell in input.verts.iter() {
        for &id in cell {
            if let Some(id) = valid_point_id(id, n) {
                fixed[id] = true;
            }
        }
    }

    let mut all_neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut line_neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut line_visits = vec![0usize; n];
    for cell in input.lines.iter() {
        let len = cell.len();
        if len == 0 {
            continue;
        }
        if let Some(id) = valid_point_id(cell[0], n) {
            fixed[id] = true;
        }
        if let Some(id) = valid_point_id(cell[len - 1], n) {
            fixed[id] = true;
        }
        for j in 1..len.saturating_sub(1) {
            if let (Some(id), Some(prev), Some(next)) = (
                valid_point_id(cell[j], n),
                valid_point_id(cell[j - 1], n),
                valid_point_id(cell[j + 1], n),
            ) {
                line_visits[id] += 1;
                line_neighbors[id].push(prev);
                line_neighbors[id].push(next);
            }
        }
    }

    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in input.polys.iter() {
        add_polygon_edges(cell, n, &mut edge_count, &mut all_neighbors);
    }
    for strip in input.strips.iter() {
        add_strip_edges(strip, n, &mut edge_count, &mut all_neighbors);
    }

    let mut restricted_neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut boundary_edge_vertex = vec![false; n];
    for (&(a, b), &count) in &edge_count {
        if count == 1 || count >= 3 {
            restricted_neighbors[a].push(b);
            restricted_neighbors[b].push(a);
            if count == 1 {
                boundary_edge_vertex[a] = true;
                boundary_edge_vertex[b] = true;
            }
        }
    }

    for nbrs in &mut all_neighbors {
        dedup_neighbors(nbrs);
    }
    for nbrs in &mut line_neighbors {
        dedup_neighbors(nbrs);
    }
    for nbrs in &mut restricted_neighbors {
        dedup_neighbors(nbrs);
    }

    let mut smooth_neighbors = vec![Vec::new(); n];
    for i in 0..n {
        if fixed[i] || line_visits[i] > 1 {
            continue;
        }

        if !line_neighbors[i].is_empty() {
            if edge_angle_allows(input, i, &line_neighbors[i]) {
                smooth_neighbors[i] = line_neighbors[i].clone();
            }
            continue;
        }

        if !restricted_neighbors[i].is_empty() {
            if (!boundary_edge_vertex[i] || !fix_boundary)
                && edge_angle_allows(input, i, &restricted_neighbors[i])
            {
                smooth_neighbors[i] = restricted_neighbors[i].clone();
            }
            continue;
        }

        smooth_neighbors[i] = all_neighbors[i].clone();
    }

    smooth_neighbors
}

fn add_polygon_edges(
    cell: &[i64],
    n: usize,
    edge_count: &mut HashMap<(usize, usize), usize>,
    all_neighbors: &mut [Vec<usize>],
) {
    let len = cell.len();
    if len < 2 {
        return;
    }
    for j in 0..len {
        if let (Some(a), Some(b)) = (
            valid_point_id(cell[j], n),
            valid_point_id(cell[(j + 1) % len], n),
        ) {
            add_edge(a, b, edge_count, all_neighbors);
        }
    }
}

fn add_strip_edges(
    strip: &[i64],
    n: usize,
    edge_count: &mut HashMap<(usize, usize), usize>,
    all_neighbors: &mut [Vec<usize>],
) {
    if strip.len() < 3 {
        return;
    }
    for tri in strip.windows(3) {
        if let (Some(a), Some(b), Some(c)) = (
            valid_point_id(tri[0], n),
            valid_point_id(tri[1], n),
            valid_point_id(tri[2], n),
        ) {
            add_edge(a, b, edge_count, all_neighbors);
            add_edge(b, c, edge_count, all_neighbors);
            add_edge(c, a, edge_count, all_neighbors);
        }
    }
}

fn add_edge(
    a: usize,
    b: usize,
    edge_count: &mut HashMap<(usize, usize), usize>,
    all_neighbors: &mut [Vec<usize>],
) {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    *edge_count.entry((lo, hi)).or_insert(0) += 1;
    all_neighbors[a].push(b);
    all_neighbors[b].push(a);
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
}

fn dedup_neighbors(nbrs: &mut Vec<usize>) {
    nbrs.sort_unstable();
    nbrs.dedup();
}

fn edge_angle_allows(input: &PolyData, point_id: usize, nbrs: &[usize]) -> bool {
    if nbrs.len() != 2 {
        return false;
    }

    let x1 = input.points.get(nbrs[0]);
    let x2 = input.points.get(point_id);
    let x3 = input.points.get(nbrs[1]);
    let mut l1 = [x2[0] - x1[0], x2[1] - x1[1], x2[2] - x1[2]];
    let mut l2 = [x3[0] - x2[0], x3[1] - x2[1], x3[2] - x2[2]];
    let n1 = (l1[0] * l1[0] + l1[1] * l1[1] + l1[2] * l1[2]).sqrt();
    let n2 = (l2[0] * l2[0] + l2[1] * l2[1] + l2[2] * l2[2]).sqrt();
    if n1 == 0.0 || n2 == 0.0 {
        return false;
    }
    for k in 0..3 {
        l1[k] /= n1;
        l2[k] /= n2;
    }
    l1[0] * l2[0] + l1[1] * l2[1] + l1[2] * l2[2] >= COS_EDGE_ANGLE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_preserves_topology() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = smooth(&pd, 5, 0.5, false);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn smooth_moves_interior() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.5, 0.5, 0.0],
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        );
        let before = pd.points.get(4);
        let result = smooth(&pd, 10, 0.5, false);
        let after = result.points.get(4);
        assert!(
            (after[0] - 1.0).abs() < (before[0] - 1.0).abs(),
            "x should move toward center"
        );
    }

    #[test]
    fn zero_iterations_noop() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = smooth(&pd, 0, 0.5, false);
        assert_eq!(result.points.get(0), pd.points.get(0));
    }

    #[test]
    fn relaxation_factor_is_not_clamped() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.5, 0.5, 0.0],
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        );

        let result = smooth(&pd, 1, 1.5, false);
        let p = result.points.get(4);
        assert!((p[0] - 1.25).abs() < 1e-12);
        assert!((p[1] - 1.25).abs() < 1e-12);
    }
}
