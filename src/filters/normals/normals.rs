use crate::data::{DataArray, PolyData};
use rayon::prelude::*;

/// Compute normals for a PolyData.
///
/// Computes cell normals from polygon winding order and averages them at
/// shared vertices to produce smooth point normals.
pub fn compute_normals(input: &PolyData) -> PolyData {
    if input.points.is_empty() || (input.polys.num_cells() == 0 && input.strips.num_cells() == 0) {
        return input.clone();
    }

    let mut output = input.clone();

    let cell_normals = compute_cell_normals(input);

    let point_normals = compute_point_normals(input, &cell_normals);

    output.point_data_mut().add_array(point_normals.into());
    output.point_data_mut().set_active_normals("Normals");

    output
}

/// Compute only cell normals (flat shading).
pub fn compute_cell_normals_only(input: &PolyData) -> PolyData {
    if input.points.is_empty() || (input.polys.num_cells() == 0 && input.strips.num_cells() == 0) {
        return input.clone();
    }

    let mut output = input.clone();

    let cell_normals = compute_cell_normals(input);
    let mut arr = DataArray::<f64>::new("Normals", 3);
    for n in &cell_normals {
        arr.push_tuple(n);
    }

    output.cell_data_mut().add_array(arr.into());
    output.cell_data_mut().set_active_normals("Normals");

    output
}

/// Compute normals using rayon parallel iteration for cell normals.
pub fn compute_normals_par(input: &PolyData) -> PolyData {
    if input.points.is_empty() || (input.polys.num_cells() == 0 && input.strips.num_cells() == 0) {
        return input.clone();
    }

    let mut output = input.clone();

    // Collect cells into a Vec so we can parallelize
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    let mut cell_normals = Vec::with_capacity(input.total_cells());
    cell_normals
        .extend((0..(input.verts.num_cells() + input.lines.num_cells())).map(|_| [1.0, 0.0, 0.0]));
    cell_normals.extend(
        cells
            .par_iter()
            .map(|cell| compute_single_cell_normal(&input.points, cell))
            .collect::<Vec<_>>(),
    );
    cell_normals.extend((0..input.strips.num_cells()).map(|_| [1.0, 0.0, 0.0]));

    let point_normals = compute_point_normals(input, &cell_normals);

    output.point_data_mut().add_array(point_normals.into());
    output.point_data_mut().set_active_normals("Normals");
    output
}

fn compute_single_cell_normal(points: &crate::data::Points<f64>, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 0.0];
    }

    let mut n = [0.0; 3];
    let mut v1 = [0.0; 3];
    let mut common_point_id = None;
    let mut point_id = 0;

    while point_id < cell.len() - 2 {
        let p0 = points.get(cell[point_id] as usize);
        let p1 = points.get(cell[point_id + 1] as usize);
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

    let p0 = points.get(cell[common_point_id] as usize);
    while point_id < cell.len() {
        let p = points.get(cell[point_id] as usize);
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

fn compute_cell_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let mut normals = Vec::with_capacity(input.total_cells());

    normals
        .extend((0..(input.verts.num_cells() + input.lines.num_cells())).map(|_| [1.0, 0.0, 0.0]));
    for cell in input.polys.iter() {
        normals.push(compute_single_cell_normal(&input.points, cell));
    }
    normals.extend((0..input.strips.num_cells()).map(|_| [1.0, 0.0, 0.0]));

    normals
}

fn compute_point_normals(input: &PolyData, cell_normals: &[[f64; 3]]) -> DataArray<f64> {
    let mut point_normals = vec![[0.0f64; 3]; input.points.len()];

    let mut cell_idx = 0;
    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            let cn = cell_normals[cell_idx];
            for &pt_id in cell {
                let pn = &mut point_normals[pt_id as usize];
                pn[0] += cn[0];
                pn[1] += cn[1];
                pn[2] += cn[2];
            }
            cell_idx += 1;
        }
    }

    let mut arr = DataArray::<f64>::new("Normals", 3);
    for pn in &point_normals {
        let len = (pn[0] * pn[0] + pn[1] * pn[1] + pn[2] * pn[2]).sqrt();
        if len > 1e-10 {
            arr.push_tuple(&[pn[0] / len, pn[1] / len, pn[2] / len]);
        } else {
            arr.push_tuple(&[0.0, 0.0, 0.0]);
        }
    }
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_triangle_normals() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_normals(&pd);

        let normals = result.point_data().normals().unwrap();
        assert_eq!(normals.num_tuples(), 3);

        // All normals should point in +Z for a CCW triangle in XY plane
        let mut buf = [0.0f64; 3];
        normals.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn shared_vertex_averaging() {
        // Two triangles sharing edge 1-2, at 90 degrees
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [0.0, 1.0, 0.0], // 2
                [0.0, 0.0, 1.0], // 3
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let result = compute_normals(&pd);

        let normals = result.point_data().normals().unwrap();
        // Point 0 is shared: its normal should be average of both face normals
        let mut buf = [0.0f64; 3];
        normals.tuple_as_f64(0, &mut buf);
        let len = (buf[0] * buf[0] + buf[1] * buf[1] + buf[2] * buf[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6, "normal should be normalized");
    }
}
