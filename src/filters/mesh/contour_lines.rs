//! Extract contour lines of scalar fields on mesh surfaces.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Extract contour lines at multiple isovalues.
pub fn multi_contour_on_mesh(mesh: &PolyData, array_name: &str, isovalues: &[f64]) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= mesh.points.len() => a,
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
            let Some(cell_ids) = valid_point_ids(cell, mesh.points.len()) else {
                continue;
            };
            let nc = cell.len();
            let mut crossings = Vec::new();
            let mut exact_vertex_points = HashMap::new();
            for i in 0..nc {
                let a = cell_ids[i];
                let b = cell_ids[(i + 1) % nc];
                let da = vals[a] - iso;
                let db = vals[b] - iso;

                if da == 0.0 {
                    let idx = *exact_vertex_points.entry(a).or_insert_with(|| {
                        let idx = all_pts.len() as i64;
                        all_pts.push(mesh.points.get(a));
                        idx
                    });
                    if !crossings.contains(&idx) {
                        crossings.push(idx);
                    }
                }
                if db == 0.0 {
                    let idx = *exact_vertex_points.entry(b).or_insert_with(|| {
                        let idx = all_pts.len() as i64;
                        all_pts.push(mesh.points.get(b));
                        idx
                    });
                    if !crossings.contains(&idx) {
                        crossings.push(idx);
                    }
                }

                if da * db < 0.0 {
                    let t = (iso - vals[a]) / (vals[b] - vals[a]);
                    let pa = mesh.points.get(a);
                    let pb = mesh.points.get(b);
                    let idx = all_pts.len() as i64;
                    all_pts.push([
                        pa[0] + t * (pb[0] - pa[0]),
                        pa[1] + t * (pb[1] - pa[1]),
                        pa[2] + t * (pb[2] - pa[2]),
                    ]);
                    crossings.push(idx);
                }
            }
            if crossings.len() >= 2 {
                for pair in crossings.chunks(2) {
                    if pair.len() == 2 {
                        all_lines.push_cell(&[pair[0], pair[1]]);
                        iso_data.push(iso);
                    }
                }
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
    if n_contours == 0 {
        return PolyData::new();
    }
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

    let isovalues: Vec<f64> = if n_contours == 1 {
        vec![min_v]
    } else {
        (0..n_contours)
            .map(|i| min_v + (max_v - min_v) * i as f64 / (n_contours - 1) as f64)
            .collect()
    };
    multi_contour_on_mesh(mesh, array_name, &isovalues)
}

/// Compute contour length for each isovalue.
pub fn contour_lengths(contours: &PolyData) -> Vec<(f64, f64)> {
    let iso_arr = match contours.cell_data().get_array("Isovalue") {
        Some(a) => a,
        None => return Vec::new(),
    };
    let mut lengths: Vec<(u64, f64, f64)> = Vec::new();
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
                let bits = buf[0].to_bits();
                if let Some((_, _, total)) = lengths.iter_mut().find(|(key, _, _)| *key == bits) {
                    *total += len;
                } else {
                    lengths.push((bits, buf[0], len));
                }
            }
        }
        ci += 1;
    }
    lengths
        .into_iter()
        .map(|(_, isovalue, length)| (isovalue, length))
        .collect()
}

fn valid_point_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    cell.iter()
        .map(|&id| usize::try_from(id).ok().filter(|&id| id < n_points))
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

    #[test]
    fn contour_through_exact_vertex() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.5],
                1,
            )));

        let result = multi_contour_on_mesh(&mesh, "f", &[0.5]);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn invalid_cell_is_skipped() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0],
                1,
            )));

        let result = multi_contour_on_mesh(&mesh, "f", &[0.5]);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn contour_lengths_preserve_close_isovalues() {
        let mut contours = PolyData::new();
        contours.points.push([0.0, 0.0, 0.0]);
        contours.points.push([1.0, 0.0, 0.0]);
        contours.points.push([0.0, 1.0, 0.0]);
        contours.points.push([0.0, 2.0, 0.0]);
        contours.lines.push_cell(&[0, 1]);
        contours.lines.push_cell(&[2, 3]);
        contours
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Isovalue",
                vec![0.0001, 0.0002],
                1,
            )));

        let lengths = contour_lengths(&contours);
        assert_eq!(lengths.len(), 2);
        assert!(lengths
            .iter()
            .any(|&(iso, len)| iso == 0.0001 && len == 1.0));
        assert!(lengths
            .iter()
            .any(|&(iso, len)| iso == 0.0002 && len == 1.0));
    }
}
