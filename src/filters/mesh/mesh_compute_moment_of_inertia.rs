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
    let axes = jacobi_eigen_symmetric([
        [moi.ixx, moi.ixy, moi.ixz],
        [moi.ixy, moi.iyy, moi.iyz],
        [moi.ixz, moi.iyz, moi.izz],
    ]);
    (axes[0], axes[1], axes[2])
}

fn jacobi_eigen_symmetric(mut a: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..50 {
        let mut p = 0usize;
        let mut q = 1usize;
        let mut max = a[0][1].abs();
        for &(r, c) in &[(0usize, 2usize), (1usize, 2usize)] {
            if a[r][c].abs() > max {
                max = a[r][c].abs();
                p = r;
                q = c;
            }
        }
        if max < 1e-12 {
            break;
        }

        let theta = 0.5 * (2.0 * a[p][q]).atan2(a[q][q] - a[p][p]);
        let c = theta.cos();
        let s = theta.sin();
        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];

        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for r in 0..3 {
            if r == p || r == q {
                continue;
            }
            let arp = a[r][p];
            let arq = a[r][q];
            a[r][p] = c * arp - s * arq;
            a[p][r] = a[r][p];
            a[r][q] = s * arp + c * arq;
            a[q][r] = a[r][q];
        }

        for row in &mut v {
            let vrp = row[p];
            let vrq = row[q];
            row[p] = c * vrp - s * vrq;
            row[q] = s * vrp + c * vrq;
        }
    }

    let mut pairs = [
        (a[0][0], normalize([v[0][0], v[1][0], v[2][0]])),
        (a[1][1], normalize([v[0][1], v[1][1], v[2][1]])),
        (a[2][2], normalize([v[0][2], v[1][2], v[2][2]])),
    ];
    pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    [pairs[0].1, pairs[1].1, pairs[2].1]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-15 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
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
