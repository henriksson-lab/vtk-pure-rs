//! Flip mesh face winding and normals.

use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

/// Flip all face windings (reverse vertex order in each polygon).
pub fn flip_faces(mesh: &PolyData) -> PolyData {
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        let mut reversed: Vec<i64> = cell.to_vec();
        reversed.reverse();
        new_polys.push_cell(&reversed);
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    // Flip normals array if present
    if let Some(normals) = result.point_data().get_array("Normals") {
        if normals.num_components() == 3 {
            let n = normals.num_tuples();
            let mut buf = [0.0f64; 3];
            let data: Vec<f64> = (0..n)
                .flat_map(|i| {
                    normals.tuple_as_f64(i, &mut buf);
                    vec![-buf[0], -buf[1], -buf[2]]
                })
                .collect();
            result
                .point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec("Normals", data, 3)));
        }
    }
    result
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
}
