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
/// Principal axes of the inertia tensor, ordered by *decreasing moment of inertia*.
///
/// Delegates to [`crate::filters::mesh::principal_axes::principal_axes`], which
/// diagonalises the covariance matrix exactly like `vtkOBBTree::ComputeOBB` and orders
/// its axes by decreasing spread. The inertia tensor shares those eigenvectors but
/// reverses the ordering, so the axes are walked backwards here.
pub fn principal_axes(mesh: &PolyData) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let (_, axes, _) = crate::filters::mesh::principal_axes::principal_axes(mesh);
    (axes[2], axes[1], axes[0])
}

/// Extents of the oriented bounding box, ordered `[max, mid, min]`.
///
/// `vtkOBBTree::ComputeOBB` diagonalises the *covariance* matrix and returns the
/// axes in decreasing eigenvalue order, i.e. widest spread first. `principal_axes`
/// diagonalises the *inertia* tensor, whose eigenvalues are ordered exactly the
/// other way round (I = trace(C)*Id - C shares the eigenvectors and reverses the
/// ordering), so the axes have to be walked in reverse to match VTK.
pub fn oriented_bounding_box_size(mesh: &PolyData) -> [f64; 3] {
    let (a3, a2, a1) = principal_axes(mesh);
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
        let axes = {
            let (a1, a2, a3) = principal_axes(&m);
            [a1, a2, a3]
        };
        let dot = |u: [f64; 3], v: [f64; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        for (i, &u) in axes.iter().enumerate() {
            assert!(
                (dot(u, u) - 1.0).abs() < 1e-10,
                "axis {i} is not unit length"
            );
            for (j, &v) in axes.iter().enumerate().skip(i + 1) {
                assert!(
                    dot(u, v).abs() < 1e-10,
                    "axes {i} and {j} are not orthogonal"
                );
            }
        }
    }
    #[test]
    fn test_obb() {
        // Flat triangle in z = 0, longest extent along x.
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = oriented_bounding_box_size(&m);
        // Sizes come back widest-first, like vtkOBBTree's max/mid/min axes.
        assert!(s[0] >= s[1] && s[1] >= s[2], "sizes not ordered: {s:?}");
        assert!(s[0] > 2.9, "widest extent should span the long edge: {s:?}");
        // The geometry is planar, so the thinnest axis has no extent at all.
        assert!(
            s[2] < 1e-10,
            "planar input should be flat on min axis: {s:?}"
        );
    }
}
