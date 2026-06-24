//! Free-form deformation (FFD) using trilinear lattice.

use crate::data::PolyData;

/// Apply free-form deformation using a control lattice.
/// lattice is a 3D grid of control points of size (lx, ly, lz).
/// Each control point is a displacement [dx, dy, dz].
pub fn ffd_deform(
    mesh: &PolyData,
    bbox_min: [f64; 3],
    bbox_max: [f64; 3],
    lattice: &[[f64; 3]],
    lx: usize,
    ly: usize,
    lz: usize,
) -> PolyData {
    let n = mesh.points.len();
    let range = [
        bbox_max[0] - bbox_min[0],
        bbox_max[1] - bbox_min[1],
        bbox_max[2] - bbox_min[2],
    ];

    let mut result = mesh.clone();
    for i in 0..n {
        let p = mesh.points.get(i);
        let s = [
            ((p[0] - bbox_min[0]) / range[0].max(1e-15)).clamp(0.0, 1.0),
            ((p[1] - bbox_min[1]) / range[1].max(1e-15)).clamp(0.0, 1.0),
            ((p[2] - bbox_min[2]) / range[2].max(1e-15)).clamp(0.0, 1.0),
        ];
        let disp = trilinear_interp(lattice, lx, ly, lz, s);
        result
            .points
            .set(i, [p[0] + disp[0], p[1] + disp[1], p[2] + disp[2]]);
    }
    result
}

/// Simpler FFD: bend mesh along one axis using a 1D spline of displacements.
pub fn bend_along_axis(mesh: &PolyData, axis: usize, displacements: &[[f64; 3]]) -> PolyData {
    let n = mesh.points.len();
    if displacements.is_empty() || n == 0 {
        return mesh.clone();
    }

    let mut mn = f64::INFINITY;
    let mut mx = f64::NEG_INFINITY;
    for i in 0..n {
        let p = mesh.points.get(i);
        mn = mn.min(p[axis]);
        mx = mx.max(p[axis]);
    }
    let range = (mx - mn).max(1e-15);
    let nd = displacements.len();

    let mut result = mesh.clone();
    for i in 0..n {
        let p = mesh.points.get(i);
        let t = (p[axis] - mn) / range;
        let fi = t * (nd - 1) as f64;
        let i0 = (fi.floor() as usize).min(nd - 2);
        let i1 = i0 + 1;
        let frac = fi - i0 as f64;
        let d = [
            displacements[i0][0] * (1.0 - frac) + displacements[i1][0] * frac,
            displacements[i0][1] * (1.0 - frac) + displacements[i1][1] * frac,
            displacements[i0][2] * (1.0 - frac) + displacements[i1][2] * frac,
        ];
        result
            .points
            .set(i, [p[0] + d[0], p[1] + d[1], p[2] + d[2]]);
    }
    result
}

fn trilinear_interp(
    lattice: &[[f64; 3]],
    lx: usize,
    ly: usize,
    lz: usize,
    s: [f64; 3],
) -> [f64; 3] {
    let fx = s[0] * (lx - 1) as f64;
    let fy = s[1] * (ly - 1) as f64;
    let fz = s[2] * (lz - 1) as f64;
    let ix = (fx.floor() as usize).min(lx - 2);
    let iy = (fy.floor() as usize).min(ly - 2);
    let iz = (fz.floor() as usize).min(lz - 2);
    let tx = fx - ix as f64;
    let ty = fy - iy as f64;
    let tz = fz - iz as f64;

    let get = |x: usize, y: usize, z: usize| -> [f64; 3] {
        let idx = x + y * lx + z * lx * ly;
        if idx < lattice.len() {
            lattice[idx]
        } else {
            [0.0; 3]
        }
    };

    let mut r = [0.0; 3];
    for d in 0..3 {
        let c000 = get(ix, iy, iz)[d];
        let c100 = get(ix + 1, iy, iz)[d];
        let c010 = get(ix, iy + 1, iz)[d];
        let c110 = get(ix + 1, iy + 1, iz)[d];
        let c001 = get(ix, iy, iz + 1)[d];
        let c101 = get(ix + 1, iy, iz + 1)[d];
        let c011 = get(ix, iy + 1, iz + 1)[d];
        let c111 = get(ix + 1, iy + 1, iz + 1)[d];
        r[d] = c000 * (1.0 - tx) * (1.0 - ty) * (1.0 - tz)
            + c100 * tx * (1.0 - ty) * (1.0 - tz)
            + c010 * (1.0 - tx) * ty * (1.0 - tz)
            + c110 * tx * ty * (1.0 - tz)
            + c001 * (1.0 - tx) * (1.0 - ty) * tz
            + c101 * tx * (1.0 - ty) * tz
            + c011 * (1.0 - tx) * ty * tz
            + c111 * tx * ty * tz;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bend() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let disps = vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let r = bend_along_axis(&mesh, 2, &disps);
        // Point at z=1 should be displaced by [1,0,0]
        let p = r.points.get(3);
        assert!((p[0] - 1.5).abs() < 1e-10);
    }
}
