//! Displacement analysis: compare mesh positions before/after deformation.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-vertex displacement vectors and magnitudes between two meshes.
pub fn displacement_analysis(original: &PolyData, deformed: &PolyData) -> PolyData {
    let n = original.points.len().min(deformed.points.len());
    let mut disp = Vec::with_capacity(n * 3);
    let mut mag = Vec::with_capacity(n);
    for i in 0..n {
        let a = original.points.get(i);
        let b = deformed.points.get(i);
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        disp.extend_from_slice(&d);
        mag.push((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    let mut result = deformed.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Displacement",
            disp,
            3,
        )));
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DisplacementMag",
            mag,
            1,
        )));
    result
}

/// Compute strain (normalized displacement relative to original edge lengths).
pub fn strain_analysis(original: &PolyData, deformed: &PolyData) -> PolyData {
    let n = original.polys.num_cells().min(deformed.polys.num_cells());
    let orig_cells: Vec<Vec<i64>> = original.polys.iter().map(|c| c.to_vec()).collect();
    let def_cells: Vec<Vec<i64>> = deformed.polys.iter().map(|c| c.to_vec()).collect();

    let mut strain_data = Vec::with_capacity(n);
    for ci in 0..n {
        let oc = &orig_cells[ci];
        let dc = &def_cells[ci];
        if oc.len() < 3 || dc.len() < 3 {
            strain_data.push(0.0);
            continue;
        }
        let mut total_strain = 0.0;
        let mut count = 0;
        let nc = oc.len();
        for i in 0..nc {
            let a = oc[i] as usize;
            let b = oc[(i + 1) % nc] as usize;
            let pa = original.points.get(a);
            let pb = original.points.get(b);
            let da = deformed.points.get(a);
            let db = deformed.points.get(b);
            let orig_len =
                ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                    .sqrt();
            let def_len =
                ((da[0] - db[0]).powi(2) + (da[1] - db[1]).powi(2) + (da[2] - db[2]).powi(2))
                    .sqrt();
            if orig_len > 1e-15 {
                total_strain += (def_len - orig_len).abs() / orig_len;
                count += 1;
            }
        }
        strain_data.push(if count > 0 {
            total_strain / count as f64
        } else {
            0.0
        });
    }

    let mut result = deformed.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Strain",
            strain_data,
            1,
        )));
    result
}

/// Compute max/mean/rms displacement statistics.
pub fn displacement_stats(mesh: &PolyData) -> (f64, f64, f64) {
    let arr = match mesh.point_data().get_array("DisplacementMag") {
        Some(a) => a,
        None => return (0.0, 0.0, 0.0),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut max_d = 0.0f64;
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        max_d = max_d.max(buf[0]);
        sum += buf[0];
        sum2 += buf[0] * buf[0];
    }
    if n > 0 {
        (max_d, sum / n as f64, (sum2 / n as f64).sqrt())
    } else {
        (0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let orig = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let def = PolyData::from_points(vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0]]);
        let result = displacement_analysis(&orig, &def);
        let (max_d, mean, _) = displacement_stats(&result);
        assert!((max_d - 1.0).abs() < 0.01);
        assert!((mean - 1.0).abs() < 0.01);
    }
    #[test]
    fn strain() {
        let orig = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let def = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        ); // scaled 2x
        let result = strain_analysis(&orig, &def);
        let arr = result.cell_data().get_array("Strain").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01); // 100% strain
    }
}
