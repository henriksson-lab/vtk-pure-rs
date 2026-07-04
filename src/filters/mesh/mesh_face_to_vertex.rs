//! Interpolate cell (face) data to vertex (point) data by averaging incident faces.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn face_to_vertex(mesh: &PolyData, cell_scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.cell_data().get_array(cell_scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let num_components = arr.num_components();
    let mut sum = vec![0.0f64; n * num_components];
    let mut count = vec![0u32; n];
    let mut buf = vec![0.0f64; num_components];
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let full_cell_data = arr.num_tuples() >= poly_offset + mesh.polys.num_cells();
    let first_poly_tuple = if full_cell_data { poly_offset } else { 0 };

    accumulate_cells(
        &mesh.polys,
        arr,
        first_poly_tuple,
        n,
        &mut buf,
        &mut sum,
        &mut count,
    );

    let strip_offset = poly_offset + mesh.polys.num_cells();
    if arr.num_tuples() >= strip_offset + mesh.strips.num_cells() {
        accumulate_cells(
            &mesh.strips,
            arr,
            strip_offset,
            n,
            &mut buf,
            &mut sum,
            &mut count,
        );
    } else if arr.num_tuples() >= mesh.polys.num_cells() + mesh.strips.num_cells() {
        accumulate_cells(
            &mesh.strips,
            arr,
            mesh.polys.num_cells(),
            n,
            &mut buf,
            &mut sum,
            &mut count,
        );
    }

    let mut vals = Vec::with_capacity(n * num_components);
    for i in 0..n {
        for c in 0..num_components {
            vals.push(if count[i] > 0 {
                sum[i * num_components + c] / count[i] as f64
            } else {
                0.0
            });
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            cell_scalar_name,
            vals,
            num_components,
        )));
    result.point_data_mut().set_active_scalars(cell_scalar_name);
    result
}

fn accumulate_cells(
    cells: &crate::data::CellArray,
    arr: &AnyDataArray,
    first_tuple: usize,
    num_points: usize,
    buf: &mut [f64],
    sum: &mut [f64],
    count: &mut [u32],
) {
    let num_components = arr.num_components();
    for (cell_id, cell) in cells.iter().enumerate() {
        let tuple_id = first_tuple + cell_id;
        if tuple_id >= arr.num_tuples() {
            break;
        }
        arr.tuple_as_f64(tuple_id, buf);
        for &point_id in cell {
            let Some(point_id) = valid_point_index(point_id, num_points) else {
                continue;
            };
            for component in 0..num_components {
                sum[point_id * num_components + component] += buf[component];
            }
            count[point_id] += 1;
        }
    }
}

fn valid_point_index(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_f2v() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cv",
                vec![10.0, 20.0],
                1,
            )));
        let r = face_to_vertex(&mesh, "cv");
        let arr = r.point_data().get_array("cv").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(1, &mut b); // vertex 1 is in both faces
        assert!((b[0] - 15.0).abs() < 1e-9); // average of 10 and 20
    }

    #[test]
    fn test_f2v_multi_component() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vec",
                vec![10.0, 20.0, 30.0, 40.0],
                2,
            )));
        let r = face_to_vertex(&mesh, "vec");
        let arr = r.point_data().get_array("vec").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut b = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut b);
        assert!((b[0] - 20.0).abs() < 1e-9);
        assert!((b[1] - 30.0).abs() < 1e-9);
    }
}
