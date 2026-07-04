//! Sort mesh cells by various criteria.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};
use crate::types::Scalar;

pub fn sort_by_area_ascending(mesh: &PolyData) -> PolyData {
    sort_cells(mesh, false)
}
pub fn sort_by_area_descending(mesh: &PolyData) -> PolyData {
    sort_cells(mesh, true)
}
fn sort_cells(mesh: &PolyData, descending: bool) -> PolyData {
    let mut cells: Vec<(usize, Vec<i64>, f64)> = mesh
        .polys
        .iter()
        .enumerate()
        .map(|cell| {
            let (cell_id, cell) = cell;
            let c = cell.to_vec();
            let area = polygon_area(mesh, &c);
            (cell_id, c, area)
        })
        .collect();
    if descending {
        cells.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        cells.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    }
    rebuild_sorted_mesh(mesh, &cells)
}

pub fn sort_by_centroid_z(mesh: &PolyData) -> PolyData {
    let mut cells: Vec<(usize, Vec<i64>, f64)> = mesh
        .polys
        .iter()
        .enumerate()
        .map(|cell| {
            let (cell_id, cell) = cell;
            let c = cell.to_vec();
            if c.is_empty() {
                return (cell_id, c, 0.0);
            }
            let mut z = 0.0;
            let mut count = 0usize;
            for &v in &c {
                if v >= 0 && (v as usize) < mesh.points.len() {
                    z += mesh.points.get(v as usize)[2];
                    count += 1;
                }
            }
            if count > 0 {
                z /= count as f64;
            }
            (cell_id, c, z)
        })
        .collect();
    cells.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    rebuild_sorted_mesh(mesh, &cells)
}

fn polygon_area(mesh: &PolyData, cell: &[i64]) -> f64 {
    if cell.len() < 3
        || cell
            .iter()
            .any(|&v| v < 0 || v as usize >= mesh.points.len())
    {
        return 0.0;
    }
    let a = mesh.points.get(cell[0] as usize);
    let mut area = 0.0;
    for i in 1..cell.len() - 1 {
        let b = mesh.points.get(cell[i] as usize);
        let c = mesh.points.get(cell[i + 1] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        area += 0.5
            * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt();
    }
    area
}

fn rebuild_sorted_mesh(mesh: &PolyData, cells: &[(usize, Vec<i64>, f64)]) -> PolyData {
    let mut polys = CellArray::new();
    let mut source_cell_ids = Vec::with_capacity(cells.len());
    for (cell_id, c, _) in cells {
        polys.push_cell(c);
        source_cell_ids.push(*cell_id);
    }
    let mut r = mesh.clone();
    r.polys = polys;
    let cell_data = copy_cell_data_by_indices(mesh, &source_cell_ids);
    *r.cell_data_mut() = cell_data;
    r
}

fn copy_cell_data_by_indices(mesh: &PolyData, indices: &[usize]) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();
    for array in mesh.cell_data().field_data().iter() {
        if array.num_tuples() == mesh.polys.num_cells() {
            output.add_array(copy_array_by_indices(array, indices));
        } else {
            output.add_array(array.clone());
        }
    }
    preserve_active_attributes(mesh.cell_data(), &mut output);
    output
}

fn preserve_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}

fn copy_array_by_indices(array: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(copy_typed_array($array, indices))
        };
    }

    match array {
        AnyDataArray::F32(array) => copy!(array, F32),
        AnyDataArray::F64(array) => copy!(array, F64),
        AnyDataArray::I8(array) => copy!(array, I8),
        AnyDataArray::I16(array) => copy!(array, I16),
        AnyDataArray::I32(array) => copy!(array, I32),
        AnyDataArray::I64(array) => copy!(array, I64),
        AnyDataArray::U8(array) => copy!(array, U8),
        AnyDataArray::U16(array) => copy!(array, U16),
        AnyDataArray::U32(array) => copy!(array, U32),
        AnyDataArray::U64(array) => copy!(array, U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * num_components);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, num_components)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn test_area() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [5.0, 10.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = sort_by_area_ascending(&m);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_area_uses_full_polygon_and_reorders_cell_data() {
        let mut m = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![vec![0, 1, 2, 3], vec![4, 5, 6]],
        );
        m.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "id",
                vec![10, 20],
                1,
            )));
        m.cell_data_mut().set_active_scalars("id");

        let r = sort_by_area_ascending(&m);
        let first = r.polys.iter().next().unwrap();
        assert_eq!(first, &[4, 5, 6]);
        let ids = r.cell_data().scalars().unwrap();
        let mut buf = [0.0];
        ids.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 20.0);
    }
    #[test]
    fn test_z() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 10.0],
                [1.0, 0.0, 10.0],
                [0.5, 1.0, 10.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = sort_by_centroid_z(&m);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
