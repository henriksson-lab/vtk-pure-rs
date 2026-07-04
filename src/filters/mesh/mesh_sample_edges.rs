//! Sample points along mesh edges.

use crate::data::{CellArray, Points, PolyData};
use std::collections::HashSet;

/// Sample N equidistant points along each edge.
pub fn sample_edges(mesh: &PolyData, samples_per_edge: usize) -> PolyData {
    let mut seen = HashSet::new();
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let s = samples_per_edge.max(1);
    sample_open_edges(&mesh.lines, mesh, s, &mut seen, &mut pts, &mut verts);
    sample_closed_edges(&mesh.polys, mesh, s, &mut seen, &mut pts, &mut verts);
    sample_strip_edges(&mesh.strips, mesh, s, &mut seen, &mut pts, &mut verts);
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}

/// Sample points at edge midpoints only.
pub fn edge_midpoints(mesh: &PolyData) -> PolyData {
    let mut seen = HashSet::new();
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    midpoint_open_edges(&mesh.lines, mesh, &mut seen, &mut pts, &mut verts);
    midpoint_closed_edges(&mesh.polys, mesh, &mut seen, &mut pts, &mut verts);
    midpoint_strip_edges(&mesh.strips, mesh, &mut seen, &mut pts, &mut verts);
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}

fn sample_open_edges(
    cells: &CellArray,
    mesh: &PolyData,
    samples_per_edge: usize,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    for cell in cells.iter() {
        for edge in cell.windows(2) {
            sample_edge(edge[0], edge[1], mesh, samples_per_edge, seen, pts, verts);
        }
    }
}

fn sample_closed_edges(
    cells: &CellArray,
    mesh: &PolyData,
    samples_per_edge: usize,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    for cell in cells.iter() {
        if cell.len() < 2 {
            continue;
        }
        for edge in cell.windows(2) {
            sample_edge(edge[0], edge[1], mesh, samples_per_edge, seen, pts, verts);
        }
        sample_edge(
            cell[cell.len() - 1],
            cell[0],
            mesh,
            samples_per_edge,
            seen,
            pts,
            verts,
        );
    }
}

fn sample_strip_edges(
    cells: &CellArray,
    mesh: &PolyData,
    samples_per_edge: usize,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    for cell in cells.iter() {
        for tri in cell.windows(3) {
            sample_edge(tri[0], tri[1], mesh, samples_per_edge, seen, pts, verts);
            sample_edge(tri[1], tri[2], mesh, samples_per_edge, seen, pts, verts);
            sample_edge(tri[2], tri[0], mesh, samples_per_edge, seen, pts, verts);
        }
    }
}

fn sample_edge(
    a: i64,
    b: i64,
    mesh: &PolyData,
    samples_per_edge: usize,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    let Some((a, b)) = valid_edge(a, b, mesh.points.len(), seen) else {
        return;
    };
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    for j in 0..=samples_per_edge {
        let t = j as f64 / samples_per_edge as f64;
        let idx = pts.len();
        pts.push([
            pa[0] + (pb[0] - pa[0]) * t,
            pa[1] + (pb[1] - pa[1]) * t,
            pa[2] + (pb[2] - pa[2]) * t,
        ]);
        verts.push_cell(&[idx as i64]);
    }
}

fn midpoint_open_edges(
    cells: &CellArray,
    mesh: &PolyData,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    for cell in cells.iter() {
        for edge in cell.windows(2) {
            push_midpoint(edge[0], edge[1], mesh, seen, pts, verts);
        }
    }
}

fn midpoint_closed_edges(
    cells: &CellArray,
    mesh: &PolyData,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    for cell in cells.iter() {
        if cell.len() < 2 {
            continue;
        }
        for edge in cell.windows(2) {
            push_midpoint(edge[0], edge[1], mesh, seen, pts, verts);
        }
        push_midpoint(cell[cell.len() - 1], cell[0], mesh, seen, pts, verts);
    }
}

fn midpoint_strip_edges(
    cells: &CellArray,
    mesh: &PolyData,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    for cell in cells.iter() {
        for tri in cell.windows(3) {
            push_midpoint(tri[0], tri[1], mesh, seen, pts, verts);
            push_midpoint(tri[1], tri[2], mesh, seen, pts, verts);
            push_midpoint(tri[2], tri[0], mesh, seen, pts, verts);
        }
    }
}

fn push_midpoint(
    a: i64,
    b: i64,
    mesh: &PolyData,
    seen: &mut HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    verts: &mut CellArray,
) {
    let Some((a, b)) = valid_edge(a, b, mesh.points.len(), seen) else {
        return;
    };
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let idx = pts.len();
    pts.push([
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ]);
    verts.push_cell(&[idx as i64]);
}

fn valid_edge(
    a: i64,
    b: i64,
    num_points: usize,
    seen: &mut HashSet<(usize, usize)>,
) -> Option<(usize, usize)> {
    let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) else {
        return None;
    };
    if a >= num_points || b >= num_points || a == b {
        return None;
    }
    let key = (a.min(b), a.max(b));
    seen.insert(key).then_some((a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sample() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = sample_edges(&mesh, 3);
        assert_eq!(r.points.len(), 12); // 3 edges * 4 points each
    }
    #[test]
    fn test_midpoints() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = edge_midpoints(&mesh);
        assert_eq!(r.points.len(), 3); // 3 edges
        let p = r.points.get(0);
        assert!((p[0] - 0.5).abs() < 1e-12);
        assert!(p[1].abs() < 1e-12);
    }

    #[test]
    fn test_samples_lines() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 1, 2]);

        let r = edge_midpoints(&mesh);
        assert_eq!(r.points.len(), 2);
    }

    #[test]
    fn test_samples_triangle_strips() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let r = edge_midpoints(&mesh);
        assert_eq!(r.points.len(), 5);
    }
}
