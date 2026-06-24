//! Classify mesh faces as inside/outside another mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn classify_faces_inside(mesh: &PolyData, reference: &PolyData) -> PolyData {
    let mut data = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.is_empty() {
            data.push(0.0);
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &v in cell {
            let p = mesh.points.get(v as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;
        cz /= n;
        let inside = point_in_mesh([cx + 1e-7, cy + 1.3e-7, cz + 0.9e-7], reference);
        data.push(if inside { 1.0 } else { 0.0 });
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Inside", data, 1)));
    r
}
fn point_in_mesh(p: [f64; 3], mesh: &PolyData) -> bool {
    let mut crossings = 0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            if ray_tri(p, a, b, c) {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}
fn ray_tri(o: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> bool {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let d = [1.0, 0.0, 0.0];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if det.abs() < 1e-12 {
        return false;
    }
    let inv = 1.0 / det;
    let s = [o[0] - a[0], o[1] - a[1], o[2] - a[2]];
    let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return false;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = inv * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]) > 1e-12
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let inner = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.05, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let outer = PolyData::from_triangles(
            vec![
                [-5.0, -5.0, -5.0],
                [5.0, -5.0, -5.0],
                [0.0, 5.0, -5.0],
                [0.0, 0.0, 5.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = classify_faces_inside(&inner, &outer);
        assert!(r.cell_data().get_array("Inside").is_some());
    }
}
