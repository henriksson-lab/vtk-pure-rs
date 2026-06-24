//! Triangulate arbitrary polygons using ear clipping.

use crate::data::{CellArray, PolyData};

/// Triangulate all polygons in a mesh using fan triangulation.
/// Works correctly for convex polygons; approximate for concave ones.
pub fn triangulate_polygons(mesh: &PolyData) -> PolyData {
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() <= 3 {
            new_polys.push_cell(cell);
        } else {
            // Fan triangulation from first vertex
            let v0 = cell[0];
            for i in 1..cell.len() - 1 {
                new_polys.push_cell(&[v0, cell[i], cell[i + 1]]);
            }
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    result
}

/// Triangulate using ear clipping (handles concave polygons better).
pub fn triangulate_ear_clip(mesh: &PolyData) -> PolyData {
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() <= 3 {
            new_polys.push_cell(cell);
            continue;
        }
        let pts: Vec<[f64; 3]> = cell.iter().map(|&v| mesh.points.get(v as usize)).collect();
        let indices: Vec<i64> = cell.to_vec();
        let tris = ear_clip_2d(&pts, &indices);
        for tri in tris {
            new_polys.push_cell(&tri);
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    result
}

fn ear_clip_2d(pts: &[[f64; 3]], indices: &[i64]) -> Vec<[i64; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![[indices[0], indices[1], indices[2]]];
    }

    // Project to 2D using dominant normal axis
    let normal = polygon_normal(pts);
    let axis = if normal[0].abs() > normal[1].abs() && normal[0].abs() > normal[2].abs() {
        0
    } else if normal[1].abs() > normal[2].abs() {
        1
    } else {
        2
    };
    let (u_axis, v_axis) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let pts2d: Vec<[f64; 2]> = pts.iter().map(|p| [p[u_axis], p[v_axis]]).collect();

    let mut remaining: Vec<usize> = (0..n).collect();
    let mut result = Vec::new();

    let mut max_iter = n * n;
    while remaining.len() > 3 && max_iter > 0 {
        max_iter -= 1;
        let m = remaining.len();
        let mut found = false;
        for i in 0..m {
            let prev = remaining[(i + m - 1) % m];
            let curr = remaining[i];
            let next = remaining[(i + 1) % m];
            if is_ear(&pts2d, &remaining, prev, curr, next) {
                result.push([indices[prev], indices[curr], indices[next]]);
                remaining.remove(i);
                found = true;
                break;
            }
        }
        if !found {
            break;
        }
    }
    if remaining.len() == 3 {
        result.push([
            indices[remaining[0]],
            indices[remaining[1]],
            indices[remaining[2]],
        ]);
    }
    result
}

fn is_ear(pts: &[[f64; 2]], remaining: &[usize], prev: usize, curr: usize, next: usize) -> bool {
    let a = pts[prev];
    let b = pts[curr];
    let c = pts[next];
    let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if cross <= 0.0 {
        return false;
    } // not convex vertex
    for &idx in remaining {
        if idx == prev || idx == curr || idx == next {
            continue;
        }
        if point_in_triangle(pts[idx], a, b, c) {
            return false;
        }
    }
    true
}

fn point_in_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

fn sign(p1: [f64; 2], p2: [f64; 2], p3: [f64; 2]) -> f64 {
    (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
}

fn polygon_normal(pts: &[[f64; 3]]) -> [f64; 3] {
    let mut n = [0.0; 3];
    for i in 0..pts.len() {
        let j = (i + 1) % pts.len();
        n[0] += (pts[i][1] - pts[j][1]) * (pts[i][2] + pts[j][2]);
        n[1] += (pts[i][2] - pts[j][2]) * (pts[i][0] + pts[j][0]);
        n[2] += (pts[i][0] - pts[j][0]) * (pts[i][1] + pts[j][1]);
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fan() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![],
        );
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        let r = triangulate_polygons(&mesh);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_ear_clip() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![],
        );
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        let r = triangulate_ear_clip(&mesh);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_triangle_passthrough() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = triangulate_polygons(&mesh);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
