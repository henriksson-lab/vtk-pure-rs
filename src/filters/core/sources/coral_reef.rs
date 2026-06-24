//! Coral reef structure (branching fractal-like).
use crate::data::{CellArray, Points, PolyData};
pub fn coral_branch(
    height: f64,
    base_r: f64,
    branches: usize,
    depth: usize,
    seed: u64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let polys = CellArray::new();
    let mut lines = CellArray::new();
    let mut rng = seed;
    grow_coral(
        &mut pts,
        &mut lines,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        height,
        base_r,
        branches,
        depth,
        &mut rng,
    );
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
fn grow_coral(
    pts: &mut Points<f64>,
    lines: &mut CellArray,
    origin: [f64; 3],
    dir: [f64; 3],
    length: f64,
    radius: f64,
    branches: usize,
    depth: usize,
    rng: &mut u64,
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
    let nb = branches.max(2);
    for bi in 0..nb {
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let angle = ((*rng >> 33) as f64 / u32::MAX as f64) * 0.8 + 0.2;
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let phi = 2.0 * std::f64::consts::PI * bi as f64 / nb as f64
            + ((*rng >> 33) as f64 / u32::MAX as f64 - 0.5) * 0.5;
        let up = if dir[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let side = normalize(cross(dir, up));
        let fwd = cross(dir, side);
        let c = angle.cos();
        let s = angle.sin();
        let new_dir = normalize([
            dir[0] * c + side[0] * s * phi.cos() + fwd[0] * s * phi.sin(),
            dir[1] * c + side[1] * s * phi.cos() + fwd[1] * s * phi.sin(),
            dir[2] * c + side[2] * s * phi.cos() + fwd[2] * s * phi.sin(),
        ]);
        *rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let len_ratio = 0.5 + 0.3 * ((*rng >> 33) as f64 / u32::MAX as f64);
        grow_coral(
            pts,
            lines,
            end,
            new_dir,
            length * len_ratio,
            radius * 0.7,
            branches,
            depth - 1,
            rng,
        );
    }
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
    fn test() {
        let c = coral_branch(3.0, 0.2, 3, 3, 42);
        assert!(c.lines.num_cells() > 10);
    }
}
