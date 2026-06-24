//! Extract sub-mesh where all vertices of a face satisfy scalar threshold.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_where_all_above(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_where(mesh, array_name, |v| v >= threshold)
}
pub fn extract_where_all_below(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_where(mesh, array_name, |v| v <= threshold)
}
pub fn extract_where_any_above(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    extract_where_any(mesh, array_name, |v| v >= threshold)
}
fn extract_where(mesh: &PolyData, array_name: &str, pred: impl Fn(f64) -> bool) -> PolyData {
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
        if cell
            .iter()
            .all(|&v| (v as usize) < vals.len() && pred(vals[v as usize]))
        {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    rebuild(mesh, &used, &kept)
}
fn extract_where_any(mesh: &PolyData, array_name: &str, pred: impl Fn(f64) -> bool) -> PolyData {
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
        if cell
            .iter()
            .any(|&v| (v as usize) < vals.len() && pred(vals[v as usize]))
        {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    rebuild(mesh, &used, &kept)
}
fn rebuild(mesh: &PolyData, used: &[bool], kept: &[Vec<i64>]) -> PolyData {
    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for c in kept {
        polys.push_cell(&c.iter().map(|&v| pm[v as usize] as i64).collect::<Vec<_>>());
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_above() {
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
                vec![1.0, 6.0, 3.0, 8.0, 9.0],
                1,
            )));
        let r = extract_where_all_above(&m, "s", 5.0);
        assert_eq!(r.polys.num_cells(), 1);
    }
    #[test]
    fn test_any() {
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
                vec![1.0, 6.0, 3.0, 8.0, 9.0],
                1,
            )));
        let r = extract_where_any_above(&m, "s", 5.0);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
