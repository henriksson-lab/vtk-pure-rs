//! Compute moments of inertia for a mesh.
use crate::data::PolyData;
pub struct MomentOfInertia {
    pub ixx: f64,
    pub iyy: f64,
    pub izz: f64,
    pub ixy: f64,
    pub ixz: f64,
    pub iyz: f64,
}
pub fn moment_of_inertia(mesh: &PolyData) -> MomentOfInertia {
    let n = mesh.points.len();
    if n == 0 {
        return MomentOfInertia {
            ixx: 0.0,
            iyy: 0.0,
            izz: 0.0,
            ixy: 0.0,
            ixz: 0.0,
            iyz: 0.0,
        };
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    let mut ixx = 0.0;
    let mut iyy = 0.0;
    let mut izz = 0.0;
    let mut ixy = 0.0;
    let mut ixz = 0.0;
    let mut iyz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - cx;
        let dy = p[1] - cy;
        let dz = p[2] - cz;
        ixx += dy * dy + dz * dz;
        iyy += dx * dx + dz * dz;
        izz += dx * dx + dy * dy;
        ixy -= dx * dy;
        ixz -= dx * dz;
        iyz -= dy * dz;
    }
    MomentOfInertia {
        ixx,
        iyy,
        izz,
        ixy,
        ixz,
        iyz,
    }
}
pub fn principal_axes(mesh: &PolyData) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let moi = moment_of_inertia(mesh);
    // Approximate principal axes via power iteration on inertia tensor
    let m = [
        [moi.ixx, moi.ixy, moi.ixz],
        [moi.ixy, moi.iyy, moi.iyz],
        [moi.ixz, moi.iyz, moi.izz],
    ];
    let mut v = [1.0, 0.0, 0.0];
    for _ in 0..50 {
        let mv = [
            m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
            m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
            m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
        ];
        let l = (mv[0] * mv[0] + mv[1] * mv[1] + mv[2] * mv[2])
            .sqrt()
            .max(1e-15);
        v = [mv[0] / l, mv[1] / l, mv[2] / l];
    }
    let axis1 = v;
    // Second axis: orthogonal
    let mut v2 = if v[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let d = v2[0] * v[0] + v2[1] * v[1] + v2[2] * v[2];
    v2 = [v2[0] - d * v[0], v2[1] - d * v[1], v2[2] - d * v[2]];
    let l = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2])
        .sqrt()
        .max(1e-15);
    v2 = [v2[0] / l, v2[1] / l, v2[2] / l];
    let axis3 = [
        axis1[1] * v2[2] - axis1[2] * v2[1],
        axis1[2] * v2[0] - axis1[0] * v2[2],
        axis1[0] * v2[1] - axis1[1] * v2[0],
    ];
    (axis1, v2, axis3)
}
pub fn oriented_bounding_box_size(mesh: &PolyData) -> [f64; 3] {
    let (a1, a2, a3) = principal_axes(mesh);
    let n = mesh.points.len();
    if n == 0 {
        return [0.0; 3];
    }
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        let d1 = p[0] * a1[0] + p[1] * a1[1] + p[2] * a1[2];
        let d2 = p[0] * a2[0] + p[1] * a2[1] + p[2] * a2[2];
        let d3 = p[0] * a3[0] + p[1] * a3[1] + p[2] * a3[2];
        mn[0] = mn[0].min(d1);
        mx[0] = mx[0].max(d1);
        mn[1] = mn[1].min(d2);
        mx[1] = mx[1].max(d2);
        mn[2] = mn[2].min(d3);
        mx[2] = mx[2].max(d3);
    }
    [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_moi() {
        let m = PolyData::from_triangles(
            vec![
                [1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, -1.0, 0.0],
            ],
            vec![[0, 2, 1], [1, 3, 0]],
        );
        let moi = moment_of_inertia(&m);
        assert!(moi.ixx > 0.0);
        assert!(moi.iyy > 0.0);
    }
    #[test]
    fn test_axes() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.1, 0.0]],
            vec![[0, 2, 1]],
        );
        let (a1, a2, a3) = principal_axes(&m);
        assert!((a1[0] * a1[0] + a1[1] * a1[1] + a1[2] * a1[2] - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_obb() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = oriented_bounding_box_size(&m);
        assert!(s[0] > 0.0);
    }
}
