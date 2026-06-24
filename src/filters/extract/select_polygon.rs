//! Select cells inside a 2D polygon outline.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Select cells whose centroids lie inside a 2D polygon (XY plane).
///
/// The polygon is defined as a closed loop of 2D points.
/// Returns a new PolyData with a "SelectedByPolygon" cell data array
/// (1.0 = inside, 0.0 = outside).
pub fn select_cells_in_polygon(mesh: &PolyData, polygon: &[[f64; 2]]) -> PolyData {
    if polygon.len() < 3 {
        return mesh.clone();
    }

    let n_cells = mesh.polys.num_cells();
    let mut selection = vec![0.0f64; n_cells];

    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.is_empty() {
            continue;
        }
        // Compute cell centroid in XY
        let mut cx = 0.0;
        let mut cy = 0.0;
        for &pid in cell {
            let p = mesh.points.get(pid as usize);
            cx += p[0];
            cy += p[1];
        }
        let n = cell.len() as f64;
        cx /= n;
        cy /= n;

        if point_in_polygon(cx, cy, polygon) {
            selection[ci] = 1.0;
        }
    }

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SelectedByPolygon",
            selection,
            1,
        )));
    result
}

/// Extract only the cells whose centroids lie inside the polygon.
pub fn extract_cells_in_polygon(mesh: &PolyData, polygon: &[[f64; 2]]) -> PolyData {
    if polygon.len() < 3 {
        return mesh.clone();
    }

    let mut new_points = crate::data::Points::<f64>::new();
    let mut new_polys = crate::data::CellArray::new();
    let mut point_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for cell in mesh.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        for &pid in cell {
            let p = mesh.points.get(pid as usize);
            cx += p[0];
            cy += p[1];
        }
        let n = cell.len() as f64;
        if !point_in_polygon(cx / n, cy / n, polygon) {
            continue;
        }

        let mut new_ids = Vec::with_capacity(cell.len());
        for &pid in cell {
            let old = pid as usize;
            let new_idx = *point_map.entry(old).or_insert_with(|| {
                let idx = new_points.len();
                new_points.push(mesh.points.get(old));
                idx
            });
            new_ids.push(new_idx as i64);
        }
        new_polys.push_cell(&new_ids);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    result
}

/// Select points inside a 2D polygon (XY plane).
pub fn select_points_in_polygon(mesh: &PolyData, polygon: &[[f64; 2]]) -> PolyData {
    if polygon.len() < 3 {
        return mesh.clone();
    }

    let n = mesh.points.len();
    let mut selection = vec![0.0f64; n];
    for i in 0..n {
        let p = mesh.points.get(i);
        if point_in_polygon(p[0], p[1], polygon) {
            selection[i] = 1.0;
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SelectedByPolygon",
            selection,
            1,
        )));
    result
}

/// Ray-casting point-in-polygon test (XY plane).
fn point_in_polygon(px: f64, py: f64, polygon: &[[f64; 2]]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let yi = polygon[i][1];
        let yj = polygon[j][1];
        if (yi > py) != (yj > py) {
            let xi = polygon[i][0];
            let xj = polygon[j][0];
            let x_intersect = xi + (py - yi) / (yj - yi) * (xj - xi);
            if px < x_intersect {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_inside_square() {
        let polygon = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        assert!(point_in_polygon(5.0, 5.0, &polygon));
        assert!(!point_in_polygon(15.0, 5.0, &polygon));
    }

    #[test]
    fn select_cells() {
        let mesh = PolyData::from_triangles(
            vec![
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [1.5, 2.0, 0.0], // inside
                [20.0, 20.0, 0.0],
                [21.0, 20.0, 0.0],
                [20.5, 21.0, 0.0], // outside
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let polygon = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let result = select_cells_in_polygon(&mesh, &polygon);
        let arr = result.cell_data().get_array("SelectedByPolygon").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // first triangle inside
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0); // second triangle outside
    }

    #[test]
    fn extract_cells() {
        let mesh = PolyData::from_triangles(
            vec![
                [1.0, 1.0, 0.0],
                [2.0, 1.0, 0.0],
                [1.5, 2.0, 0.0],
                [20.0, 20.0, 0.0],
                [21.0, 20.0, 0.0],
                [20.5, 21.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let polygon = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let result = extract_cells_in_polygon(&mesh, &polygon);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn select_points() {
        let mesh = PolyData::from_points(vec![[1.0, 1.0, 0.0], [5.0, 5.0, 0.0], [20.0, 20.0, 0.0]]);
        let polygon = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let result = select_points_in_polygon(&mesh, &polygon);
        let arr = result.point_data().get_array("SelectedByPolygon").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0); // outside
    }
}
