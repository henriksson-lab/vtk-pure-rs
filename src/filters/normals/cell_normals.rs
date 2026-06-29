use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-cell normals and add them as cell data.
///
/// For each polygon, computes the face normal using VTK's polygon normal
/// accumulation. The result is the active 3-component "Normals" array in cell
/// data.
pub fn compute_cell_normals(input: &PolyData) -> PolyData {
    let mut normals = Vec::with_capacity(input.total_cells() * 3);

    for _ in 0..(input.verts.num_cells() + input.lines.num_cells()) {
        normals.extend_from_slice(&[1.0, 0.0, 0.0]);
    }
    for cell in input.polys.iter() {
        normals.extend_from_slice(&compute_single_cell_normal(input, cell));
    }
    for _ in 0..input.strips.num_cells() {
        normals.extend_from_slice(&[1.0, 0.0, 0.0]);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Normals", normals, 3,
        )));
    pd.cell_data_mut().set_active_normals("Normals");
    pd
}

fn compute_single_cell_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 0.0];
    }

    let mut n = [0.0; 3];
    let mut v1 = [0.0; 3];
    let mut common_point_id = None;
    let mut point_id = 0;

    while point_id < cell.len() - 2 {
        let p0 = input.points.get(cell[point_id] as usize);
        let p1 = input.points.get(cell[point_id + 1] as usize);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if squared_norm(v1) > 0.0 {
            common_point_id = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_point_id) = common_point_id else {
        return n;
    };
    if point_id >= cell.len() {
        return n;
    }

    let p0 = input.points.get(cell[common_point_id] as usize);
    while point_id < cell.len() {
        let p = input.points.get(cell[point_id] as usize);
        let v2 = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let cross = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        n[0] += cross[0];
        n[1] += cross[1];
        n[2] += cross[2];
        v1 = v2;
        point_id += 1;
    }

    let len = squared_norm(n).sqrt();
    if len != 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        n
    }
}

fn squared_norm(v: [f64; 3]) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
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
        let arr = result.cell_data().normals().unwrap();
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
        let arr = result.cell_data().normals().unwrap();
        assert_eq!(arr.num_tuples(), 2);
    }

    #[test]
    fn cell_order_includes_default_line_normals() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.lines.push_cell(&[0, 1]);

        let result = compute_cell_normals(&pd);
        let arr = result.cell_data().normals().unwrap();
        assert_eq!(arr.num_tuples(), 2);

        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert_eq!(val, [1.0, 0.0, 0.0]);
        arr.tuple_as_f64(1, &mut val);
        assert!(val[2] > 0.9, "nz = {}", val[2]);
    }
}
