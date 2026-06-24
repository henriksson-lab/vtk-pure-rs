//! Fractal tree (recursive branching L-system-like).
use crate::data::{CellArray, Points, PolyData};
pub fn fractal_tree(
    trunk_length: f64,
    trunk_radius: f64,
    branch_ratio: f64,
    angle_degrees: f64,
    depth: usize,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    grow_branch(
        &mut pts,
        &mut lines,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        trunk_length,
        trunk_radius,
        branch_ratio,
        angle_degrees.to_radians(),
        depth,
    );
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
fn grow_branch(
    pts: &mut Points<f64>,
    lines: &mut CellArray,
    origin: [f64; 3],
    dir: [f64; 3],
    length: f64,
    _radius: f64,
    ratio: f64,
    angle: f64,
    depth: usize,
) {
    let end = [
        origin[0] + dir[0] * length,
        origin[1] + dir[1] * length,
        origin[2] + dir[2] * length,
    ];
    let ob = pts.len();
    pts.push(origin);
    pts.push(end);
    lines.push_cell(&[ob as i64, (ob + 1) as i64]);
    if depth == 0 {
        return;
    }
    // Two branches
    let new_len = length * ratio;
    // Branch 1: rotate around X-like axis
    let up = if dir[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let side = normalize(cross(dir, up));
    let c = angle.cos();
    let s = angle.sin();
    let d1 = [
        dir[0] * c + side[0] * s,
        dir[1] * c + side[1] * s,
        dir[2] * c + side[2] * s,
    ];
    let d1 = normalize(d1);
    grow_branch(
        pts,
        lines,
        end,
        d1,
        new_len,
        _radius * ratio,
        ratio,
        angle,
        depth - 1,
    );
    // Branch 2: opposite side
    let d2 = [
        dir[0] * c - side[0] * s,
        dir[1] * c - side[1] * s,
        dir[2] * c - side[2] * s,
    ];
    let d2 = normalize(d2);
    grow_branch(
        pts,
        lines,
        end,
        d2,
        new_len,
        _radius * ratio,
        ratio,
        angle,
        depth - 1,
    );
    // Branch 3: perpendicular
    let perp = cross(dir, side);
    let perp = normalize(perp);
    let d3 = [
        dir[0] * c + perp[0] * s,
        dir[1] * c + perp[1] * s,
        dir[2] * c + perp[2] * s,
    ];
    let d3 = normalize(d3);
    grow_branch(
        pts,
        lines,
        end,
        d3,
        new_len,
        _radius * ratio,
        ratio,
        angle,
        depth - 1,
    );
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_shallow() {
        let t = fractal_tree(5.0, 0.3, 0.7, 30.0, 2);
        assert!(t.lines.num_cells() > 5);
    }
    #[test]
    fn test_deep() {
        let t = fractal_tree(5.0, 0.3, 0.65, 25.0, 4);
        assert!(t.lines.num_cells() > 40);
    }
}
