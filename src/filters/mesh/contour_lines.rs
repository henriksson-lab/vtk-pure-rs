//! Extract contour lines of scalar fields on mesh surfaces.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Extract contour lines at multiple isovalues.
pub fn multi_contour_on_mesh(mesh: &PolyData, array_name: &str, isovalues: &[f64]) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut all_pts = Points::<f64>::new();
    let mut all_lines = CellArray::new();
    let mut iso_data = Vec::new();

    for &iso in isovalues {
        for cell in mesh.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let nc = cell.len();
            let mut crossings = Vec::new();
            for i in 0..nc {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % nc] as usize;
                if (vals[a] - iso) * (vals[b] - iso) < 0.0 {
                    let t = (iso - vals[a]) / (vals[b] - vals[a]);
                    let pa = mesh.points.get(a);
                    let pb = mesh.points.get(b);
                    crossings.push([
                        pa[0] + t * (pb[0] - pa[0]),
                        pa[1] + t * (pb[1] - pa[1]),
                        pa[2] + t * (pb[2] - pa[2]),
                    ]);
                }
            }
            if crossings.len() >= 2 {
                let i0 = all_pts.len() as i64;
                all_pts.push(crossings[0]);
                let i1 = all_pts.len() as i64;
                all_pts.push(crossings[1]);
                all_lines.push_cell(&[i0, i1]);
                iso_data.push(iso);
            }
        }
    }

    let mut result = PolyData::new();
    result.points = all_pts;
    result.lines = all_lines;
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Isovalue", iso_data, 1,
        )));
    result
}

/// Extract contour lines at regular intervals.
pub fn contour_lines_regular(mesh: &PolyData, array_name: &str, n_contours: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        min_v = min_v.min(buf[0]);
        max_v = max_v.max(buf[0]);
    }
    if (max_v - min_v).abs() < 1e-15 {
        return PolyData::new();
    }

    let isovalues: Vec<f64> = (1..=n_contours)
        .map(|i| min_v + (max_v - min_v) * i as f64 / (n_contours + 1) as f64)
        .collect();
    multi_contour_on_mesh(mesh, array_name, &isovalues)
}

/// Compute contour length for each isovalue.
pub fn contour_lengths(contours: &PolyData) -> Vec<(f64, f64)> {
    let iso_arr = match contours.cell_data().get_array("Isovalue") {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut lengths: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
    let mut buf = [0.0f64];
    let mut ci = 0;
    for cell in contours.lines.iter() {
        if cell.len() >= 2 {
            let a = contours.points.get(cell[0] as usize);
            let b = contours.points.get(cell[1] as usize);
            let len =
                ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            if ci < iso_arr.num_tuples() {
                iso_arr.tuple_as_f64(ci, &mut buf);
                *lengths.entry((buf[0] * 1000.0) as i64).or_insert(0.0) += len;
            }
        }
        ci += 1;
    }
    lengths
        .into_iter()
        .map(|(k, v)| (k as f64 / 1000.0, v))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multi_iso() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                (0..25).map(|i| i as f64).collect(),
                1,
            )));
        let result = multi_contour_on_mesh(&mesh, "f", &[5.0, 10.0, 15.0]);
        assert!(result.lines.num_cells() > 0);
        assert!(result.cell_data().get_array("Isovalue").is_some());
    }
    #[test]
    fn regular_contours() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                (0..25).map(|i| i as f64).collect(),
                1,
            )));
        let result = contour_lines_regular(&mesh, "f", 4);
        assert!(result.lines.num_cells() > 0);
    }
}
