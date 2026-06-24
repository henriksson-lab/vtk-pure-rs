use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-cell normals and add them as cell data.
///
/// For each polygon, computes the face normal using Newell's method.
/// The result is a 3-component "CellNormals" array in cell data.
pub fn compute_cell_normals(input: &PolyData) -> PolyData {
    let mut normals = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
            continue;
        }

        // Newell's method
        let mut nx = 0.0;
        let mut ny = 0.0;
        let mut nz = 0.0;
        let n = cell.len();
        for i in 0..n {
            let p = input.points.get(cell[i] as usize);
            let q = input.points.get(cell[(i + 1) % n] as usize);
            nx += (p[1] - q[1]) * (p[2] + q[2]);
            ny += (p[2] - q[2]) * (p[0] + q[0]);
            nz += (p[0] - q[0]) * (p[1] + q[1]);
        }
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 1e-20 {
            normals.extend_from_slice(&[nx / len, ny / len, nz / len]);
        } else {
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CellNormals",
            normals,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_triangle_normal() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_cell_normals(&pd);
        let arr = result.cell_data().get_array("CellNormals").unwrap();
        assert_eq!(arr.num_tuples(), 1);
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        // Normal should point in +z
        assert!(val[2] > 0.9, "nz = {}", val[2]);
    }

    #[test]
    fn two_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let result = compute_cell_normals(&pd);
        let arr = result.cell_data().get_array("CellNormals").unwrap();
        assert_eq!(arr.num_tuples(), 2);
    }
}
