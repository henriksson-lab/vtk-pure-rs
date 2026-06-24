//! Compute surface area grouped by cell data label.
use crate::data::PolyData;
pub fn area_by_label(mesh: &PolyData, label_array: &str) -> std::collections::HashMap<usize, f64> {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return std::collections::HashMap::new(),
    };
    let mut buf = [0.0f64];
    let mut result = std::collections::HashMap::new();
    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 {
            continue;
        }
        arr.tuple_as_f64(ci, &mut buf);
        let label = buf[0] as usize;
        let a = mesh.points.get(cell[0] as usize);
        let mut area = 0.0;
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            area += 0.5
                * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                    + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                    + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                .sqrt();
        }
        *result.entry(label).or_insert(0.0) += area;
    }
    result
}
pub fn total_area(mesh: &PolyData) -> f64 {
    let mut total = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            total += 0.5
                * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                    + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                    + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                .sqrt();
        }
    }
    total
}
pub fn face_count_by_label(
    mesh: &PolyData,
    label_array: &str,
) -> std::collections::HashMap<usize, usize> {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) if a.num_components() == 1 => a,
        _ => return std::collections::HashMap::new(),
    };
    let mut buf = [0.0f64];
    let mut result = std::collections::HashMap::new();
    for ci in 0..mesh.polys.num_cells() {
        arr.tuple_as_f64(ci, &mut buf);
        *result.entry(buf[0] as usize).or_insert(0) += 1;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_total() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((total_area(&m) - 2.0).abs() < 1e-10);
    }
    #[test]
    fn test_by_label() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        m.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "L",
                vec![0.0, 1.0],
                1,
            )));
        let areas = area_by_label(&m, "L");
        assert_eq!(areas.len(), 2);
    }
}
