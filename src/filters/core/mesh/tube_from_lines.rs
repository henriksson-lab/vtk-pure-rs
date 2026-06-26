//! Generate tube geometry around polylines.

use crate::data::{CellArray, Points, PolyData};

/// Generate a tube around each line in the mesh.
pub fn tube_from_lines(mesh: &PolyData, radius: f64, sides: usize) -> PolyData {
    let sides = sides.max(3);
    let mut pts = Points::<f64>::new();
    let mut strips = CellArray::new();

    for line in mesh.lines.iter() {
        if line.len() < 2 {
            continue;
        }
        let line_pts: Vec<[f64; 3]> = line
            .iter()
            .map(|&id| mesh.points.get(id as usize))
            .collect();
        let seg_count = line_pts.len() - 1;
        let ring_start = pts.len();

        for (si, _) in line_pts.iter().enumerate() {
            // Compute tangent
            let tangent = if si == 0 {
                sub(line_pts[1], line_pts[0])
            } else if si == seg_count {
                sub(line_pts[si], line_pts[si - 1])
            } else {
                sub(line_pts[si + 1], line_pts[si - 1])
            };
            let t = normalize(tangent);
            let (n1, n2) = perpendicular_frame(t);

            for j in 0..sides {
                let angle = 2.0 * std::f64::consts::PI * j as f64 / sides as f64;
                let c = angle.cos();
                let s = angle.sin();
                pts.push([
                    line_pts[si][0] + radius * (c * n1[0] + s * n2[0]),
                    line_pts[si][1] + radius * (c * n1[1] + s * n2[1]),
                    line_pts[si][2] + radius * (c * n1[2] + s * n2[2]),
                ]);
            }
        }

        // VTK's vtkTubeFilter emits one triangle strip per side when sides share vertices.
        for j in 0..sides {
            let j0 = j;
            let j1 = (j + 1) % sides;
            let mut strip = Vec::with_capacity(line_pts.len() * 2);
            for si in 0..line_pts.len() {
                let ring = ring_start + si * sides;
                strip.push((ring + j1) as i64);
                strip.push((ring + j0) as i64);
            }
            strips.push_cell(&strip);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.strips = strips;
    result
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-15 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn perpendicular_frame(t: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let up = if t[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let n1 = normalize(cross(t, up));
    let n2 = cross(t, n1);
    (n1, n2)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tube() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 1, 2]);
        let result = tube_from_lines(&mesh, 0.1, 8);
        assert_eq!(result.points.len(), 24); // 3 rings * 8 sides
        assert_eq!(result.strips.num_cells(), 8); // one side strip per side
        assert!(result.strips.iter().all(|strip| strip.len() == 6)); // 3 line points * 2
    }
    #[test]
    fn test_tube_triangle() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([0.0, 0.0, 5.0]);
        mesh.lines.push_cell(&[0, 1]);
        let result = tube_from_lines(&mesh, 1.0, 3);
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.strips.num_cells(), 3);
        assert!(result.strips.iter().all(|strip| strip.len() == 4));
    }
}
