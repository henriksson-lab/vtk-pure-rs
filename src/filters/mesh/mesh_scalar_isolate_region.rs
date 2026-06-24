//! Isolate mesh regions where scalar is within a threshold band.
use crate::data::{CellArray, Points, PolyData};
pub fn isolate_by_scalar_band(mesh: &PolyData, array_name: &str, lo: f64, hi: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    for cell in mesh.polys.iter() {
        let all_in = cell.iter().all(|&v| {
            let vi = v as usize;
            vi < vals.len() && vals[vi] >= lo && vals[vi] <= hi
        });
        if all_in {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        polys.push_cell(&c.iter().map(|&v| pm[v as usize] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn isolate_above_threshold(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    isolate_by_scalar_band(mesh, array_name, threshold, f64::INFINITY)
}
pub fn isolate_below_threshold(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    isolate_by_scalar_band(mesh, array_name, f64::NEG_INFINITY, threshold)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_band() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0, 8.0, 9.0],
                1,
            )));
        let r = isolate_by_scalar_band(&m, "s", 0.0, 5.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_above() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![5.0, 6.0, 7.0],
                1,
            )));
        let r = isolate_above_threshold(&m, "s", 4.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
