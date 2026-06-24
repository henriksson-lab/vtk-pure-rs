//! Extract iso-surface (contour) of a scalar field defined on mesh vertices.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_isoline_on_mesh(mesh: &PolyData, array_name: &str, isovalue: f64) -> PolyData {
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
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let nc = cell.len();
        let mut edge_pts: Vec<[f64; 3]> = Vec::new();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a >= vals.len() || b >= vals.len() {
                continue;
            }
            let va = vals[a];
            let vb = vals[b];
            if (va - isovalue) * (vb - isovalue) < 0.0 {
                let t = (isovalue - va) / (vb - va);
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                edge_pts.push([
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    pa[2] + t * (pb[2] - pa[2]),
                ]);
            }
        }
        if edge_pts.len() == 2 {
            let i0 = pts.len();
            pts.push(edge_pts[0]);
            pts.push(edge_pts[1]);
            lines.push_cell(&[i0 as i64, (i0 + 1) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn extract_multiple_isolines(mesh: &PolyData, array_name: &str, values: &[f64]) -> PolyData {
    let mut all_pts = Points::<f64>::new();
    let mut all_lines = CellArray::new();
    for &v in values {
        let iso = extract_isoline_on_mesh(mesh, array_name, v);
        let base = all_pts.len() as i64;
        for i in 0..iso.points.len() {
            all_pts.push(iso.points.get(i));
        }
        for cell in iso.lines.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            all_lines.push_cell(&shifted);
        }
    }
    let mut r = PolyData::new();
    r.points = all_pts;
    r.lines = all_lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_iso() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 3.0, 1.0],
                1,
            )));
        let r = extract_isoline_on_mesh(&m, "s", 1.5);
        assert!(r.lines.num_cells() >= 1);
    }
    #[test]
    fn test_multi() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [1.5, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 3.0, 1.5],
                1,
            )));
        let r = extract_multiple_isolines(&m, "s", &[0.5, 1.0, 2.0]);
        assert!(r.lines.num_cells() >= 2);
    }
}
