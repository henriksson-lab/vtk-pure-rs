//! Spin image descriptor for 3D shape matching.
use crate::data::PolyData;
pub fn compute_spin_image(
    mesh: &PolyData,
    vertex: usize,
    bin_size: f64,
    image_width: usize,
) -> Vec<f64> {
    let n = mesh.points.len();
    if vertex >= n {
        return vec![];
    }
    let iw = image_width.max(2);
    if bin_size <= 0.0 || !bin_size.is_finite() {
        return vec![0.0; iw * iw];
    }
    let p = mesh.points.get(vertex);
    let nm = calc_nm(mesh);
    let ni = nm[vertex];
    let mut img = vec![0.0f64; iw * iw];
    for i in 0..n {
        if i == vertex {
            continue;
        }
        let q = mesh.points.get(i);
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        let beta = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2]; // height along normal
        let perp = [
            d[0] - beta * ni[0],
            d[1] - beta * ni[1],
            d[2] - beta * ni[2],
        ];
        let alpha = (perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt(); // radial distance
        let ai = (alpha / bin_size).floor() as isize;
        let bi = (beta / bin_size + iw as f64 / 2.0).floor() as isize;
        if ai < 0 || bi < 0 || ai >= iw as isize || bi >= iw as isize {
            continue;
        }
        let ai = ai as usize;
        let bi = bi as usize;
        img[ai * iw + bi] += 1.0;
    }
    img
}
pub fn spin_image_correlation(img_a: &[f64], img_b: &[f64]) -> f64 {
    let n = img_a.len().min(img_b.len());
    if n == 0 {
        return 0.0;
    }
    let ma = img_a.iter().sum::<f64>() / n as f64;
    let mb = img_b.iter().sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut da = 0.0;
    let mut db = 0.0;
    for i in 0..n {
        let a = img_a[i] - ma;
        let b = img_b[i] - mb;
        num += a * b;
        da += a * a;
        db += b * b;
    }
    if da < 1e-30 || db < 1e-30 {
        0.0
    } else {
        num / (da * db).sqrt()
    }
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a_id = cell[0];
        let b_id = cell[1];
        let c_id = cell[2];
        if a_id < 0 || b_id < 0 || c_id < 0 {
            continue;
        }
        let a_idx = a_id as usize;
        let b_idx = b_id as usize;
        let c_idx = c_id as usize;
        if a_idx >= n || b_idx >= n || c_idx >= n {
            continue;
        }
        let a = mesh.points.get(a_idx);
        let b = mesh.points.get(b_idx);
        let c = mesh.points.get(c_idx);
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
    fn test_img() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let img = compute_spin_image(&m, 0, 0.5, 8);
        assert_eq!(img.len(), 64);
    }
    #[test]
    fn test_corr() {
        let a = vec![1.0, 0.0, 0.0, 1.0];
        let b = vec![1.0, 0.0, 0.0, 1.0];
        let c = spin_image_correlation(&a, &b);
        assert!((c - 1.0).abs() < 1e-10);
    }
}
