//! Generate ribbon surfaces from polylines.

use crate::data::{CellArray, Points, PolyData};

/// Generate a flat ribbon of given width centered on each polyline.
pub fn ribbon_from_lines(mesh: &PolyData, width: f64, up: [f64; 3]) -> PolyData {
    let half = width * 0.5;
    let up_n = normalize(up);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for line in mesh.lines.iter() {
        if line.len() < 2 {
            continue;
        }
        let line_pts: Vec<[f64; 3]> = line
            .iter()
            .map(|&id| mesh.points.get(id as usize))
            .collect();
        let start_idx = pts.len();

        for (i, p) in line_pts.iter().enumerate() {
            let tangent = if i == 0 {
                sub(line_pts[1], line_pts[0])
            } else if i == line_pts.len() - 1 {
                sub(line_pts[i], line_pts[i - 1])
            } else {
                sub(line_pts[i + 1], line_pts[i - 1])
            };
            let t = normalize(tangent);
            let mut side = cross(t, up_n);
            if length(side) < 1e-15 {
                let fallback = if t[0].abs() < 0.9 {
                    [1.0, 0.0, 0.0]
                } else {
                    [0.0, 1.0, 0.0]
                };
                side = cross(t, fallback);
            }
            let side = normalize(side);
            pts.push([
                p[0] - half * side[0],
                p[1] - half * side[1],
                p[2] - half * side[2],
            ]);
            pts.push([
                p[0] + half * side[0],
                p[1] + half * side[1],
                p[2] + half * side[2],
            ]);
        }

        for i in 0..line_pts.len() - 1 {
            let base = start_idx + i * 2;
            polys.push_cell(&[
                base as i64,
                (base + 2) as i64,
                (base + 3) as i64,
                (base + 1) as i64,
            ]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = length(v);
    if len < 1e-15 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
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
    fn test_ribbon() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.lines.push_cell(&[0, 1, 2]);
        let result = ribbon_from_lines(&mesh, 0.5, [0.0, 0.0, 1.0]);
        assert_eq!(result.points.len(), 6); // 3 points * 2 sides
        assert_eq!(result.polys.num_cells(), 2); // 2 quads
    }
    #[test]
    fn test_ribbon_vertical() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([0.0, 0.0, 1.0]);
        mesh.lines.push_cell(&[0, 1]);
        let result = ribbon_from_lines(&mesh, 1.0, [0.0, 1.0, 0.0]);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
