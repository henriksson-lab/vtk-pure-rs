//! Extract faces where cell scalar is within a range.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_faces_by_cell_scalar(
    mesh: &PolyData,
    array_name: &str,
    lo: f64,
    hi: f64,
) -> PolyData {
    let arr = match mesh.cell_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    for (ci, cell) in cells.iter().enumerate() {
        arr.tuple_as_f64(ci, &mut buf);
        if buf[0] >= lo && buf[0] <= hi {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.clone());
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
pub fn extract_faces_above_scalar(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_faces_by_cell_scalar(mesh, array_name, threshold, f64::INFINITY)
}
pub fn extract_faces_below_scalar(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_faces_by_cell_scalar(mesh, array_name, f64::NEG_INFINITY, threshold)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
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
        m.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "q",
                vec![0.5, 0.9],
                1,
            )));
        let r = extract_faces_by_cell_scalar(&m, "q", 0.0, 0.6);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
