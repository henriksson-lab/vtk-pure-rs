//! Operations on face groups: split, merge, relabel, filter by size.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Split a mesh into separate PolyData objects per face group.
pub fn split_by_label(mesh: &PolyData, label_array: &str) -> Vec<(i64, PolyData)> {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut buf = [0.0f64];
    let mut groups: std::collections::BTreeMap<i64, Vec<usize>> = std::collections::BTreeMap::new();
    for ci in 0..all_cells.len() {
        if ci < arr.num_tuples() {
            arr.tuple_as_f64(ci, &mut buf);
            groups.entry(buf[0] as i64).or_default().push(ci);
        }
    }
    groups
        .into_iter()
        .map(|(label, cells)| {
            let mut pts = Points::<f64>::new();
            let mut polys = CellArray::new();
            let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
            for &ci in &cells {
                let cell = &all_cells[ci];
                let mut ids = Vec::new();
                for &pid in cell {
                    let old = pid as usize;
                    let idx = *pm.entry(old).or_insert_with(|| {
                        let i = pts.len();
                        pts.push(mesh.points.get(old));
                        i
                    });
                    ids.push(idx as i64);
                }
                polys.push_cell(&ids);
            }
            let mut r = PolyData::new();
            r.points = pts;
            r.polys = polys;
            (label, r)
        })
        .collect()
}

/// Remove face groups with fewer than min_faces faces.
pub fn remove_small_groups(mesh: &PolyData, label_array: &str, min_faces: usize) -> PolyData {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let mut counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        *counts.entry(buf[0] as i64).or_insert(0) += 1;
    }

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let keep: Vec<usize> = (0..all_cells.len())
        .filter(|&ci| {
            if ci < arr.num_tuples() {
                arr.tuple_as_f64(ci, &mut buf);
                counts.get(&(buf[0] as i64)).copied().unwrap_or(0) >= min_faces
            } else {
                false
            }
        })
        .collect();

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut new_labels = Vec::new();
    for &ci in &keep {
        let cell = &all_cells[ci];
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pm.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
        arr.tuple_as_f64(ci, &mut buf);
        new_labels.push(buf[0]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            label_array,
            new_labels,
            1,
        )));
    result
}

/// Relabel groups sequentially starting from 0.
pub fn relabel_groups(mesh: &PolyData, label_array: &str) -> PolyData {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut buf = [0.0f64];
    let mut mapping: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    let mut next = 0;
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let old = buf[0] as i64;
            let new = *mapping.entry(old).or_insert_with(|| {
                let n = next;
                next += 1;
                n
            });
            new as f64
        })
        .collect();

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(label_array, data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "g",
                vec![1.0, 2.0],
                1,
            )));
        let parts = split_by_label(&mesh, "g");
        assert_eq!(parts.len(), 2);
    }
    #[test]
    fn remove_small() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [4, 5, 6]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "g",
                vec![1.0, 1.0, 2.0],
                1,
            )));
        let result = remove_small_groups(&mesh, "g", 2);
        assert_eq!(result.polys.num_cells(), 2); // only group 1
    }
    #[test]
    fn relabel() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "g",
                vec![10.0, 20.0],
                1,
            )));
        let result = relabel_groups(&mesh, "g");
        let arr = result.cell_data().get_array("g").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
