//! Flip mesh face winding and normals.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};

/// Flip all face windings (reverse vertex order in each polygon).
pub fn flip_faces(mesh: &PolyData) -> PolyData {
    let mut result = mesh.clone();
    result.verts = reverse_cell_array(&mesh.verts);
    result.lines = reverse_cell_array(&mesh.lines);
    result.polys = reverse_cell_array(&mesh.polys);
    result.strips = reverse_cell_array(&mesh.strips);
    if let Some(flipped) = flipped_normals(normals_array(result.point_data())) {
        result.point_data_mut().add_array(flipped);
    }
    if let Some(flipped) = flipped_normals(normals_array(result.cell_data())) {
        result.cell_data_mut().add_array(flipped);
    }
    result
}

fn normals_array(attrs: &DataSetAttributes) -> Option<&AnyDataArray> {
    attrs.normals().or_else(|| attrs.get_array("Normals"))
}

fn reverse_cell_array(cells: &CellArray) -> CellArray {
    let mut reversed_cells = CellArray::new();
    for cell in cells.iter() {
        let mut reversed: Vec<i64> = cell.to_vec();
        reversed.reverse();
        reversed_cells.push_cell(&reversed);
    }
    reversed_cells
}

/// Flip only faces whose normal points away from a given direction.
pub fn flip_faces_toward(mesh: &PolyData, direction: [f64; 3]) -> PolyData {
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            new_polys.push_cell(cell);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let n = face_normal(a, b, c);
        let dot = n[0] * direction[0] + n[1] * direction[1] + n[2] * direction[2];
        if dot < 0.0 {
            let mut reversed: Vec<i64> = cell.to_vec();
            reversed.reverse();
            new_polys.push_cell(&reversed);
        } else {
            new_polys.push_cell(cell);
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    result
}

fn face_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ]
}

fn flipped_normals(normals: Option<&AnyDataArray>) -> Option<AnyDataArray> {
    let normals = normals?;
    if normals.num_components() != 3 {
        return None;
    }

    let mut buf = [0.0f64; 3];
    let data: Vec<f64> = (0..normals.num_tuples())
        .flat_map(|i| {
            normals.tuple_as_f64(i, &mut buf);
            [-buf[0], -buf[1], -buf[2]]
        })
        .collect();
    Some(AnyDataArray::F64(DataArray::from_vec(
        normals.name(),
        data,
        3,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_flip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let flipped = flip_faces(&mesh);
        let cell: Vec<i64> = flipped.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell, vec![2, 1, 0]);
    }

    #[test]
    fn flip_reverses_all_cell_arrays() {
        let mut mesh = PolyData::new();
        mesh.points = crate::data::Points::from(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]);
        mesh.lines.push_cell(&[0, 1, 2]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let flipped = flip_faces(&mesh);
        assert_eq!(flipped.lines.iter().next().unwrap(), &[2, 1, 0]);
        assert_eq!(flipped.polys.iter().next().unwrap(), &[3, 2, 1, 0]);
        assert_eq!(flipped.strips.iter().next().unwrap(), &[3, 2, 1, 0]);
    }

    #[test]
    fn test_flip_toward() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Normal points +Z, asking for -Z should flip
        let flipped = flip_faces_toward(&mesh, [0.0, 0.0, -1.0]);
        let cell: Vec<i64> = flipped.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell, vec![2, 1, 0]);
        // Asking for +Z should keep
        let kept = flip_faces_toward(&mesh, [0.0, 0.0, 1.0]);
        let cell2: Vec<i64> = kept.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell2, vec![0, 1, 2]);
    }

    #[test]
    fn flip_reverses_point_and_cell_normals() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0],
                3,
            )));

        let flipped = flip_faces(&mesh);
        let mut buf = [0.0f64; 3];
        flipped
            .point_data()
            .get_array("Normals")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, -1.0]);
        flipped
            .cell_data()
            .get_array("Normals")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn flip_uses_active_normals_before_normals_name() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                3,
            )));
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "custom_normals",
                vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                3,
            )));
        mesh.point_data_mut().set_active_normals("custom_normals");

        let flipped = flip_faces(&mesh);
        let mut buf = [0.0f64; 3];
        flipped
            .point_data()
            .normals()
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [-1.0, 0.0, 0.0]);
        flipped
            .point_data()
            .get_array("Normals")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0, 1.0]);
    }
}
