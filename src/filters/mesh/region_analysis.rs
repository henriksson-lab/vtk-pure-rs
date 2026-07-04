//! Region analysis: compute area/volume/centroid per labeled region.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};
use crate::types::Scalar;

/// Per-region statistics from a cell data label array.
pub fn region_statistics(mesh: &PolyData, label_array: &str) -> Table {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return Table::new(),
    };
    let mut buf = [0.0f64];
    let mut regions: std::collections::BTreeMap<i64, (f64, [f64; 3], usize)> =
        std::collections::BTreeMap::new();
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();

    for (ci, cell) in mesh.polys.iter().enumerate() {
        let cell_id = poly_offset + ci;
        if cell_id >= arr.num_tuples() {
            break;
        }
        arr.tuple_as_f64(cell_id, &mut buf);
        let label = buf[0] as i64;
        if !cell_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        let (area, centroid) = cell_area_centroid(mesh, cell);
        let entry = regions.entry(label).or_insert((0.0, [0.0; 3], 0));
        entry.0 += area;
        for c in 0..3 {
            entry.1[c] += centroid[c] * area;
        }
        entry.2 += 1;
    }

    let mut label_col = Vec::new();
    let mut area_col = Vec::new();
    let mut count_col = Vec::new();
    let mut cx_col = Vec::new();
    let mut cy_col = Vec::new();
    let mut cz_col = Vec::new();

    for (&label, &(area, weighted_centroid, count)) in &regions {
        label_col.push(label as f64);
        area_col.push(area);
        count_col.push(count as f64);
        if area > 1e-15 {
            cx_col.push(weighted_centroid[0] / area);
            cy_col.push(weighted_centroid[1] / area);
            cz_col.push(weighted_centroid[2] / area);
        } else {
            cx_col.push(0.0);
            cy_col.push(0.0);
            cz_col.push(0.0);
        }
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "RegionId", label_col, 1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Area", area_col, 1)))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CellCount",
            count_col,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CentroidX",
            cx_col,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CentroidY",
            cy_col,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "CentroidZ",
            cz_col,
            1,
        )))
}

/// Extract the region with a specific label as a separate PolyData.
pub fn extract_region(mesh: &PolyData, label_array: &str, label: i64) -> PolyData {
    let arr = match mesh.cell_data().get_array(label_array) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    let mut buf = [0.0f64];
    let selected: Vec<usize> = (0..all_cells.len())
        .filter(|&ci| {
            let cell_id = poly_offset + ci;
            if cell_id < arr.num_tuples() {
                arr.tuple_as_f64(cell_id, &mut buf);
                buf[0] as i64 == label
            } else {
                false
            }
        })
        .collect();
    extract_cells(mesh, &all_cells, &selected, poly_offset)
}

fn cell_area_centroid(mesh: &PolyData, cell: &[i64]) -> (f64, [f64; 3]) {
    if cell.len() < 3 {
        return (0.0, [0.0; 3]);
    }

    let a = mesh.points.get(cell[0] as usize);
    let mut area_sum = 0.0;
    let mut weighted_centroid = [0.0; 3];
    for i in 1..cell.len() - 1 {
        let b = mesh.points.get(cell[i] as usize);
        let c = mesh.points.get(cell[i + 1] as usize);
        let area = triangle_area(a, b, c);
        let centroid = [
            (a[0] + b[0] + c[0]) / 3.0,
            (a[1] + b[1] + c[1]) / 3.0,
            (a[2] + b[2] + c[2]) / 3.0,
        ];
        area_sum += area;
        for j in 0..3 {
            weighted_centroid[j] += centroid[j] * area;
        }
    }

    if area_sum > 1e-15 {
        (
            area_sum,
            [
                weighted_centroid[0] / area_sum,
                weighted_centroid[1] / area_sum,
                weighted_centroid[2] / area_sum,
            ],
        )
    } else {
        (0.0, cell_centroid(mesh, cell))
    }
}

fn triangle_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
    .sqrt()
}
fn cell_centroid(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut c = [0.0; 3];
    for &pid in cell {
        let p = mesh.points.get(pid as usize);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let k = cell.len() as f64;
    [c[0] / k, c[1] / k, c[2] / k]
}
fn extract_cells(
    mesh: &PolyData,
    all_cells: &[Vec<i64>],
    selected: &[usize],
    cell_offset: usize,
) -> PolyData {
    let mut pts = crate::data::Points::<f64>::new();
    let mut polys = crate::data::CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut point_ids = Vec::new();
    let mut cell_ids = Vec::new();
    for &ci in selected {
        let cell = &all_cells[ci];
        if !cell_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        let mut ids = Vec::new();
        for &pid in cell {
            let old = pid as usize;
            let idx = *pm.entry(old).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(old));
                point_ids.push(old);
                i
            });
            ids.push(idx as i64);
        }
        polys.push_cell(&ids);
        cell_ids.push(cell_offset + ci);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    copy_selected_arrays(mesh.point_data(), r.point_data_mut(), &point_ids);
    copy_selected_arrays(mesh.cell_data(), r.cell_data_mut(), &cell_ids);
    r
}

fn cell_ids_are_valid(cell: &[i64], num_points: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

fn copy_selected_arrays(
    input: &crate::data::DataSetAttributes,
    output: &mut crate::data::DataSetAttributes,
    ids: &[usize],
) {
    for array in input.field_data().iter() {
        if ids.iter().all(|&id| id < array.num_tuples()) {
            let name = array.name().to_string();
            output.add_array(copy_array_tuples(array, ids));
            copy_active_attribute(input, output, &name);
        }
    }
}

fn copy_active_attribute(
    source: &crate::data::DataSetAttributes,
    target: &mut crate::data::DataSetAttributes,
    name: &str,
) {
    if source.scalars().map(|a| a.name()) == Some(name) {
        target.set_active_scalars(name);
    }
    if source.vectors().map(|a| a.name()) == Some(name) {
        target.set_active_vectors(name);
    }
    if source.normals().map(|a| a.name()) == Some(name) {
        target.set_active_normals(name);
    }
    if source.tcoords().map(|a| a.name()) == Some(name) {
        target.set_active_tcoords(name);
    }
    if source.tensors().map(|a| a.name()) == Some(name) {
        target.set_active_tensors(name);
    }
    if source.global_ids().map(|a| a.name()) == Some(name) {
        target.set_active_global_ids(name);
    }
    if source.pedigree_ids().map(|a| a.name()) == Some(name) {
        target.set_active_pedigree_ids(name);
    }
    if source.edge_flags().map(|a| a.name()) == Some(name) {
        target.set_active_edge_flags(name);
    }
    if source.tangents().map(|a| a.name()) == Some(name) {
        target.set_active_tangents(name);
    }
    if source.rational_weights().map(|a| a.name()) == Some(name) {
        target.set_active_rational_weights(name);
    }
    if source.higher_order_degrees().map(|a| a.name()) == Some(name) {
        target.set_active_higher_order_degrees(name);
    }
    if source.process_ids().map(|a| a.name()) == Some(name) {
        target.set_active_process_ids(name);
    }
}

fn copy_array_tuples(array: &AnyDataArray, ids: &[usize]) -> AnyDataArray {
    match array {
        AnyDataArray::F32(a) => AnyDataArray::F32(copy_data_array_tuples(a, ids)),
        AnyDataArray::F64(a) => AnyDataArray::F64(copy_data_array_tuples(a, ids)),
        AnyDataArray::I8(a) => AnyDataArray::I8(copy_data_array_tuples(a, ids)),
        AnyDataArray::I16(a) => AnyDataArray::I16(copy_data_array_tuples(a, ids)),
        AnyDataArray::I32(a) => AnyDataArray::I32(copy_data_array_tuples(a, ids)),
        AnyDataArray::I64(a) => AnyDataArray::I64(copy_data_array_tuples(a, ids)),
        AnyDataArray::U8(a) => AnyDataArray::U8(copy_data_array_tuples(a, ids)),
        AnyDataArray::U16(a) => AnyDataArray::U16(copy_data_array_tuples(a, ids)),
        AnyDataArray::U32(a) => AnyDataArray::U32(copy_data_array_tuples(a, ids)),
        AnyDataArray::U64(a) => AnyDataArray::U64(copy_data_array_tuples(a, ids)),
    }
}

fn copy_data_array_tuples<T: Scalar>(array: &DataArray<T>, ids: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(ids.len() * num_components);
    for &id in ids {
        data.extend_from_slice(array.tuple(id));
    }
    DataArray::from_vec(array.name(), data, num_components)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stats() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![1.0, 2.0],
                1,
            )));
        let table = region_statistics(&mesh, "region");
        assert_eq!(table.num_rows(), 2);
    }
    #[test]
    fn extract() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![1.0, 2.0],
                1,
            )));
        let r = extract_region(&mesh, "region", 1);
        assert_eq!(r.polys.num_cells(), 1);
    }

    #[test]
    fn polygon_region_area_uses_full_fan() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![7.0],
                1,
            )));

        let table = region_statistics(&mesh, "region");
        let area = table.value_f64(0, "Area").unwrap();
        assert!((area - 1.0).abs() < 1e-12);
    }

    #[test]
    fn invalid_cell_ids_are_skipped() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.polys.push_cell(&[-1, 0, 1]);
        mesh.polys.push_cell(&[0, 1, 99]);
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![1.0, 1.0, 1.0],
                1,
            )));

        let table = region_statistics(&mesh, "region");
        assert_eq!(table.num_rows(), 1);
        let r = extract_region(&mesh, "region", 1);
        assert_eq!(r.polys.num_cells(), 1);
        let region = r.cell_data().get_array("region").unwrap();
        assert_eq!(region.num_tuples(), 1);
    }

    #[test]
    fn region_labels_use_polydata_cell_order() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "region",
                vec![100.0, 101.0, 7.0],
                1,
            )));
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "quality",
                vec![1.0, 2.0, 42.0],
                1,
            )));

        let table = region_statistics(&mesh, "region");
        assert_eq!(table.num_rows(), 1);
        assert_eq!(table.value_f64(0, "RegionId").unwrap(), 7.0);

        let region = extract_region(&mesh, "region", 7);
        assert_eq!(region.polys.num_cells(), 1);
        let quality = region.cell_data().get_array("quality").unwrap();
        let mut value = [0.0];
        quality.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 42.0);
    }
}
