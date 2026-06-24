use crate::data::{CellArray, DataArray, PolyData};
use rayon::prelude::*;

/// Compute normals for a PolyData.
///
/// Computes cell normals from polygon winding order and averages them at
/// shared vertices to produce smooth point normals.
pub fn compute_normals(input: &PolyData) -> PolyData {
    let mut output = input.clone();

    let cell_normals = compute_cell_normals(&input.points, &input.polys);

    let point_normals = compute_point_normals(input.points.len(), &input.polys, &cell_normals);

    output.point_data_mut().add_array(point_normals.into());
    output.point_data_mut().set_active_normals("Normals");

    output
}

/// Compute only cell normals (flat shading).
pub fn compute_cell_normals_only(input: &PolyData) -> PolyData {
    let mut output = input.clone();

    let cell_normals = compute_cell_normals(&input.points, &input.polys);
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
    let mut output = input.clone();

    // Collect cells into a Vec so we can parallelize
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    let cell_normals: Vec<[f64; 3]> = cells
        .par_iter()
        .map(|cell| compute_single_cell_normal(&input.points, cell))
        .collect();

    let point_normals = compute_point_normals(input.points.len(), &input.polys, &cell_normals);

    output.point_data_mut().add_array(point_normals.into());
    output.point_data_mut().set_active_normals("Normals");
    output
}

fn compute_single_cell_normal(points: &crate::data::Points<f64>, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0, 0.0, 1.0];
    }
    let mut nx = 0.0;
    let mut ny = 0.0;
    let mut nz = 0.0;
    let n = cell.len();
    for i in 0..n {
        let pi = points.get(cell[i] as usize);
        let pj = points.get(cell[(i + 1) % n] as usize);
        nx += (pi[1] - pj[1]) * (pi[2] + pj[2]);
        ny += (pi[2] - pj[2]) * (pi[0] + pj[0]);
        nz += (pi[0] - pj[0]) * (pi[1] + pj[1]);
    }
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-10 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn compute_cell_normals(points: &crate::data::Points<f64>, polys: &CellArray) -> Vec<[f64; 3]> {
    let mut normals = Vec::with_capacity(polys.num_cells());

    for cell in polys.iter() {
        normals.push(compute_single_cell_normal(points, cell));
    }

    normals
}

fn compute_point_normals(
    num_points: usize,
    polys: &CellArray,
    cell_normals: &[[f64; 3]],
) -> DataArray<f64> {
    let mut point_normals = vec![[0.0f64; 3]; num_points];

    for (cell_idx, cell) in polys.iter().enumerate() {
        let cn = cell_normals[cell_idx];
        for &pt_id in cell {
            let pn = &mut point_normals[pt_id as usize];
            pn[0] += cn[0];
            pn[1] += cn[1];
            pn[2] += cn[2];
        }
    }

    let mut arr = DataArray::<f64>::new("Normals", 3);
    for pn in &point_normals {
        let len = (pn[0] * pn[0] + pn[1] * pn[1] + pn[2] * pn[2]).sqrt();
        if len > 1e-10 {
            arr.push_tuple(&[pn[0] / len, pn[1] / len, pn[2] / len]);
        } else {
            arr.push_tuple(&[0.0, 0.0, 1.0]);
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
