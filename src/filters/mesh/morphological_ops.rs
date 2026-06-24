//! Morphological operations on mesh connectivity: dilate, erode face selections.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Dilate a face selection: add neighboring faces.
pub fn dilate_face_selection(
    mesh: &PolyData,
    selection_array: &str,
    iterations: usize,
) -> PolyData {
    let arr = match mesh.cell_data().get_array(selection_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = all_cells.len();
    let mut buf = [0.0f64];
    let mut selected: Vec<bool> = (0..nc)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0] > 0.5
            } else {
                false
            }
        })
        .collect();

    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    for _ in 0..iterations {
        let mut new_sel = selected.clone();
        for ci in 0..nc {
            if !selected[ci] {
                continue;
            }
            let cell = &all_cells[ci];
            let n = cell.len();
            for i in 0..n {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % n] as usize;
                if let Some(nbs) = edge_faces.get(&(a.min(b), a.max(b))) {
                    for &ni in nbs {
                        new_sel[ni] = true;
                    }
                }
            }
        }
        selected = new_sel;
    }

    let data: Vec<f64> = selected
        .iter()
        .map(|&s| if s { 1.0 } else { 0.0 })
        .collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            selection_array,
            data,
            1,
        )));
    result
}

/// Erode a face selection: remove faces on the boundary of the selection.
pub fn erode_face_selection(mesh: &PolyData, selection_array: &str, iterations: usize) -> PolyData {
    let arr = match mesh.cell_data().get_array(selection_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = all_cells.len();
    let mut buf = [0.0f64];
    let mut selected: Vec<bool> = (0..nc)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0] > 0.5
            } else {
                false
            }
        })
        .collect();

    let mut edge_faces: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in all_cells.iter().enumerate() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            edge_faces.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }

    for _ in 0..iterations {
        let mut new_sel = selected.clone();
        for ci in 0..nc {
            if !selected[ci] {
                continue;
            }
            let cell = &all_cells[ci];
            let cn = cell.len();
            let mut is_boundary = false;
            for i in 0..cn {
                let a = cell[i] as usize;
                let b = cell[(i + 1) % cn] as usize;
                if let Some(nbs) = edge_faces.get(&(a.min(b), a.max(b))) {
                    if nbs.len() == 1 || nbs.iter().any(|&ni| !selected[ni]) {
                        is_boundary = true;
                        break;
                    }
                } else {
                    is_boundary = true;
                    break;
                }
            }
            if is_boundary {
                new_sel[ci] = false;
            }
        }
        selected = new_sel;
    }

    let data: Vec<f64> = selected
        .iter()
        .map(|&s| if s { 1.0 } else { 0.0 })
        .collect();
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            selection_array,
            data,
            1,
        )));
    result
}

/// Open: erode then dilate (removes small selected regions).
pub fn open_face_selection(mesh: &PolyData, sel: &str, r: usize) -> PolyData {
    let eroded = erode_face_selection(mesh, sel, r);
    dilate_face_selection(&eroded, sel, r)
}

/// Close: dilate then erode (fills small gaps in selection).
pub fn close_face_selection(mesh: &PolyData, sel: &str, r: usize) -> PolyData {
    let dilated = dilate_face_selection(mesh, sel, r);
    erode_face_selection(&dilated, sel, r)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dilate() {
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
        let mut sel = vec![0.0f64; 32];
        sel[0] = 1.0; // select first face
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("sel", sel, 1)));
        let result = dilate_face_selection(&mesh, "sel", 1);
        let arr = result.cell_data().get_array("sel").unwrap();
        let mut count = 0;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        assert!(count > 1); // should have grown
    }
    #[test]
    fn erode() {
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
        let sel = vec![1.0f64; 32]; // all selected
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("sel", sel, 1)));
        let result = erode_face_selection(&mesh, "sel", 1);
        let arr = result.cell_data().get_array("sel").unwrap();
        let mut count = 0;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        assert!(count < 32); // should have shrunk
    }
}
