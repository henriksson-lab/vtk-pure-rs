//! Per-face quality metrics: skewness, Jacobian ratio, warpage.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-triangle quality metrics and add as cell data.
pub fn face_quality_metrics(mesh: &PolyData) -> PolyData {
    let mut aspect = Vec::new();
    let mut min_angle = Vec::new();
    let mut max_angle = Vec::new();
    let mut area_data = Vec::new();

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            aspect.push(0.0);
            min_angle.push(0.0);
            max_angle.push(0.0);
            area_data.push(0.0);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let ab = elen(a, b);
        let bc = elen(b, c);
        let ca = elen(c, a);
        let max_e = ab.max(bc).max(ca);
        let min_e = ab.min(bc).min(ca);
        aspect.push(if min_e > 1e-15 {
            max_e / min_e
        } else {
            f64::MAX.min(1e6)
        });

        let angles = [
            angle_at(b, a, c, a),
            angle_at(a, b, c, b),
            angle_at(a, c, b, c),
        ];
        min_angle.push(angles.iter().cloned().fold(f64::MAX, f64::min).to_degrees());
        max_angle.push(angles.iter().cloned().fold(0.0f64, f64::max).to_degrees());

        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        area_data.push(
            0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt(),
        );
    }

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AspectRatio",
            aspect,
            1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MinAngle", min_angle, 1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MaxAngle", max_angle, 1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Area", area_data, 1)));
    result
}

/// Compute quality summary statistics.
pub fn quality_summary(mesh: &PolyData) -> String {
    let with_q = face_quality_metrics(mesh);
    let ar = with_q.cell_data().get_array("AspectRatio");
    let ma = with_q.cell_data().get_array("MinAngle");
    let n = mesh.polys.num_cells();
    if n == 0 {
        return "Empty mesh".to_string();
    }

    let mut buf = [0.0f64];
    let (mut ar_min, mut ar_max, mut ar_sum) = (f64::MAX, 0.0f64, 0.0);
    let (mut ma_min, mut ma_max, mut ma_sum) = (f64::MAX, 0.0f64, 0.0);
    if let Some(a) = ar {
        for i in 0..n {
            a.tuple_as_f64(i, &mut buf);
            ar_min = ar_min.min(buf[0]);
            ar_max = ar_max.max(buf[0]);
            ar_sum += buf[0];
        }
    }
    if let Some(a) = ma {
        for i in 0..n {
            a.tuple_as_f64(i, &mut buf);
            ma_min = ma_min.min(buf[0]);
            ma_max = ma_max.max(buf[0]);
            ma_sum += buf[0];
        }
    }

    format!("Faces: {n}, AR: [{ar_min:.2},{ar_max:.2}] mean={:.2}, MinAngle: [{ma_min:.1}°,{ma_max:.1}°] mean={:.1}°",
        ar_sum/n as f64, ma_sum/n as f64)
}

fn elen(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
fn angle_at(p1: [f64; 3], vertex: [f64; 3], p2: [f64; 3], _v: [f64; 3]) -> f64 {
    let u = [p1[0] - vertex[0], p1[1] - vertex[1], p1[2] - vertex[2]];
    let v = [p2[0] - vertex[0], p2[1] - vertex[1], p2[2] - vertex[2]];
    let lu = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    let lv = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if lu * lv < 1e-15 {
        return 0.0;
    }
    ((u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (lu * lv))
        .clamp(-1.0, 1.0)
        .acos()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equilateral() {
        let s3 = 3.0f64.sqrt() / 2.0;
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, s3, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = face_quality_metrics(&mesh);
        let arr = result.cell_data().get_array("AspectRatio").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01); // equilateral has AR=1
    }
    #[test]
    fn summary() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = quality_summary(&mesh);
        assert!(s.contains("Faces: 1"));
    }
}
