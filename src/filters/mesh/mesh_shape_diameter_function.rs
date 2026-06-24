//! Shape Diameter Function (SDF) for thickness estimation.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn shape_diameter_function(mesh: &PolyData, num_rays: usize, seed: u64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let nm = calc_nm(mesh);
    let nr = num_rays.max(1);
    let mut rng = seed;
    let mut sdf = vec![0.0f64; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        let ni = nm[i];
        let mut sum = 0.0;
        let mut count = 0;
        // Cast rays in cone around -normal direction
        for _ in 0..nr {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = ((rng >> 33) as f64 / u32::MAX as f64) * 0.6 - 0.3;
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let v = ((rng >> 33) as f64 / u32::MAX as f64) * 0.6 - 0.3;
            let dir = [-ni[0] + u, -ni[1] + v, -ni[2]];
            let dl = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2])
                .sqrt()
                .max(1e-15);
            let dir = [dir[0] / dl, dir[1] / dl, dir[2] / dl];
            // Cast ray against all triangles
            let mut best_t = f64::INFINITY;
            for cell in mesh.polys.iter() {
                if cell.len() < 3 {
                    continue;
                }
                let a = mesh.points.get(cell[0] as usize);
                for ci in 1..cell.len() - 1 {
                    let b = mesh.points.get(cell[ci] as usize);
                    let c = mesh.points.get(cell[ci + 1] as usize);
                    if let Some(t) = ray_tri(p, dir, a, b, c) {
                        if t > 0.01 && t < best_t {
                            best_t = t;
                        }
                    }
                }
            }
            if best_t < f64::INFINITY {
                sum += best_t;
                count += 1;
            }
        }
        sdf[i] = if count > 0 { sum / count as f64 } else { 0.0 };
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("SDF", sdf, 1)));
    r.point_data_mut().set_active_scalars("SDF");
    r
}
fn ray_tri(o: [f64; 3], d: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let s = [o[0] - a[0], o[1] - a[1], o[2] - a[2]];
    let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = inv * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    if t > 1e-12 {
        Some(t)
    } else {
        None
    }
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                nm[vi][0] += fn_[0];
                nm[vi][1] += fn_[1];
                nm[vi][2] += fn_[2];
            }
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = shape_diameter_function(&m, 3, 42);
        assert!(r.point_data().get_array("SDF").is_some());
    }
}
