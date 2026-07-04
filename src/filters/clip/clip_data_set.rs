use std::collections::HashMap;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, Points, UnstructuredGrid};
use crate::types::CellType;
use crate::types::Scalar;

#[derive(Clone, Copy)]
enum TupleSource {
    Copy(usize),
    Interpolate { a: usize, b: usize, t: f64 },
}

/// Clip an UnstructuredGrid by a plane defined by a point and normal.
///
/// Keeps cells in the half-space where `dot(p - origin, normal) > 0`.
/// Linear line and surface cells that cross the plane are split.
pub fn clip_data_set(
    input: &UnstructuredGrid,
    origin: [f64; 3],
    normal: [f64; 3],
) -> UnstructuredGrid {
    let n_points = input.points.len();

    // Classify each point
    let dists: Vec<f64> = (0..n_points)
        .map(|i| {
            let p = input.points.get(i);
            (p[0] - origin[0]) * normal[0]
                + (p[1] - origin[1]) * normal[1]
                + (p[2] - origin[2]) * normal[2]
        })
        .collect();

    let mut point_map: HashMap<usize, usize> = HashMap::new();
    let mut edge_locator: HashMap<(i64, i64), i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut point_sources = Vec::new();
    let mut cell_sources = Vec::new();
    let mut out = UnstructuredGrid::new();

    let n_cells = input.cells().num_cells();
    for ci in 0..n_cells {
        let pts = input.cell_points(ci);
        let ct = input.cell_type(ci);
        if !cell_has_valid_points(pts, n_points) {
            continue;
        }

        // Keep intact cells directly, like vtkClipDataSet does before invoking cell clipping.
        let all_inside = pts.iter().all(|&id| dists[id as usize] > 0.0);
        if all_inside {
            let remapped = remap_existing_points(
                pts,
                &mut point_map,
                &input.points,
                &mut out_points,
                &mut point_sources,
            );
            push_output_cell(&mut out, &mut cell_sources, ct, &remapped, ci);
            continue;
        }

        let any_inside = pts.iter().any(|&id| dists[id as usize] > 0.0);
        if !any_inside {
            continue;
        }

        match ct {
            CellType::Line => {
                let clipped = clip_line_cell(
                    pts,
                    &dists,
                    &mut point_map,
                    &mut edge_locator,
                    input,
                    &mut out_points,
                    &mut point_sources,
                );
                if clipped.len() == 2 {
                    push_output_cell(&mut out, &mut cell_sources, CellType::Line, &clipped, ci);
                }
            }
            CellType::Triangle | CellType::Quad | CellType::Polygon => {
                let clipped = clip_linear_cell(
                    pts,
                    &dists,
                    &mut point_map,
                    &mut edge_locator,
                    input,
                    &mut out_points,
                    &mut point_sources,
                );
                if clipped.len() >= 3 {
                    for i in 1..clipped.len() - 1 {
                        push_output_cell(
                            &mut out,
                            &mut cell_sources,
                            CellType::Triangle,
                            &[clipped[0], clipped[i], clipped[i + 1]],
                            ci,
                        );
                    }
                }
            }
            CellType::Tetra => {
                clip_tetra_cell(
                    pts,
                    &dists,
                    &mut point_map,
                    &mut edge_locator,
                    input,
                    &mut out_points,
                    &mut point_sources,
                    &mut out,
                    &mut cell_sources,
                    ci,
                );
            }
            _ => {}
        }
    }

    out.points = out_points;
    copy_point_data(input, &point_sources, &mut out);
    copy_cell_data(input, &cell_sources, &mut out);
    out
}

fn cell_has_valid_points(ids: &[i64], n_points: usize) -> bool {
    ids.iter().all(|&id| id >= 0 && (id as usize) < n_points)
}

fn push_output_cell(
    out: &mut UnstructuredGrid,
    cell_sources: &mut Vec<usize>,
    cell_type: CellType,
    point_ids: &[i64],
    source_cell_id: usize,
) {
    out.push_cell(cell_type, point_ids);
    cell_sources.push(source_cell_id);
}

fn remap_existing_points(
    ids: &[i64],
    point_map: &mut HashMap<usize, usize>,
    input_points: &Points<f64>,
    out_points: &mut Points<f64>,
    point_sources: &mut Vec<TupleSource>,
) -> Vec<i64> {
    ids.iter()
        .map(|&id| {
            let uid = id as usize;
            *point_map.entry(uid).or_insert_with(|| {
                let idx = out_points.len();
                out_points.push(input_points.get(uid));
                point_sources.push(TupleSource::Copy(uid));
                idx
            }) as i64
        })
        .collect()
}

fn get_or_insert_intersection(
    a: i64,
    b: i64,
    da: f64,
    db: f64,
    input: &UnstructuredGrid,
    out_points: &mut Points<f64>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
    point_sources: &mut Vec<TupleSource>,
) -> i64 {
    let edge_key = if a < b { (a, b) } else { (b, a) };
    if let Some(&id) = edge_locator.get(&edge_key) {
        return id;
    }

    let t = da / (da - db);
    let pa = input.points.get(a as usize);
    let pb = input.points.get(b as usize);
    let p = [
        pa[0] + t * (pb[0] - pa[0]),
        pa[1] + t * (pb[1] - pa[1]),
        pa[2] + t * (pb[2] - pa[2]),
    ];
    let id = out_points.len() as i64;
    out_points.push(p);
    point_sources.push(TupleSource::Interpolate {
        a: a as usize,
        b: b as usize,
        t,
    });
    edge_locator.insert(edge_key, id);
    id
}

fn clip_linear_cell(
    ids: &[i64],
    dists: &[f64],
    point_map: &mut HashMap<usize, usize>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
    input: &UnstructuredGrid,
    out_points: &mut Points<f64>,
    point_sources: &mut Vec<TupleSource>,
) -> Vec<i64> {
    let mut result = Vec::new();
    for i in 0..ids.len() {
        let j = (i + 1) % ids.len();
        let current = ids[i];
        let next = ids[j];
        let current_dist = dists[current as usize];
        let next_dist = dists[next as usize];
        let current_inside = current_dist > 0.0;
        let next_inside = next_dist > 0.0;

        if current_inside && !next_inside {
            result.push(get_or_insert_intersection(
                current,
                next,
                current_dist,
                next_dist,
                input,
                out_points,
                edge_locator,
                point_sources,
            ));
        } else if !current_inside && next_inside {
            result.push(get_or_insert_intersection(
                current,
                next,
                current_dist,
                next_dist,
                input,
                out_points,
                edge_locator,
                point_sources,
            ));
            let mapped =
                remap_existing_points(&[next], point_map, &input.points, out_points, point_sources);
            result.push(mapped[0]);
        } else if next_inside {
            let mapped =
                remap_existing_points(&[next], point_map, &input.points, out_points, point_sources);
            result.push(mapped[0]);
        }
    }
    result
}

fn clip_tetra_cell(
    ids: &[i64],
    dists: &[f64],
    point_map: &mut HashMap<usize, usize>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
    input: &UnstructuredGrid,
    out_points: &mut Points<f64>,
    point_sources: &mut Vec<TupleSource>,
    out: &mut UnstructuredGrid,
    cell_sources: &mut Vec<usize>,
    source_cell_id: usize,
) {
    if ids.len() != 4 {
        return;
    }

    const CASE_MASK: [usize; 4] = [1, 2, 4, 8];
    const EDGES: [[usize; 2]; 6] = [[0, 1], [1, 2], [2, 0], [0, 3], [1, 3], [2, 3]];
    const TETRA_CASES: [[i32; 7]; 16] = [
        [0, 0, 0, 0, 0, 0, 0],
        [4, 0, 3, 2, 100, 0, 0],
        [4, 0, 1, 4, 101, 0, 0],
        [6, 101, 1, 4, 100, 2, 3],
        [4, 1, 2, 5, 102, 0, 0],
        [6, 102, 5, 1, 100, 3, 0],
        [6, 102, 2, 5, 101, 0, 4],
        [6, 3, 4, 5, 100, 101, 102],
        [4, 3, 4, 5, 103, 0, 0],
        [6, 103, 4, 5, 100, 0, 2],
        [6, 103, 5, 3, 101, 1, 0],
        [6, 100, 101, 103, 2, 1, 5],
        [6, 2, 102, 1, 3, 103, 4],
        [6, 0, 1, 4, 100, 102, 103],
        [6, 0, 3, 2, 101, 103, 102],
        [4, 100, 101, 102, 103, 0, 0],
    ];

    let mut index = 0;
    for i in 0..4 {
        if dists[ids[i] as usize] > 0.0 {
            index |= CASE_MASK[i];
        }
    }

    let tetra_case = &TETRA_CASES[index];
    let n_case_points = tetra_case[0] as usize;
    if n_case_points == 0 {
        return;
    }

    let mut case_points = Vec::with_capacity(n_case_points);
    for &entry in &tetra_case[1..=n_case_points] {
        let out_id = if entry >= 100 {
            let point_id = ids[(entry - 100) as usize];
            remap_existing_points(
                &[point_id],
                point_map,
                &input.points,
                out_points,
                point_sources,
            )[0]
        } else {
            let edge = EDGES[entry as usize];
            let a = ids[edge[0]];
            let b = ids[edge[1]];
            get_or_insert_intersection(
                a,
                b,
                dists[a as usize],
                dists[b as usize],
                input,
                out_points,
                edge_locator,
                point_sources,
            )
        };
        case_points.push(out_id);
    }

    match n_case_points {
        4 => push_output_cell(
            out,
            cell_sources,
            CellType::Tetra,
            &case_points,
            source_cell_id,
        ),
        6 => push_output_cell(
            out,
            cell_sources,
            CellType::Wedge,
            &case_points,
            source_cell_id,
        ),
        _ => unreachable!(),
    }
}

fn clip_line_cell(
    ids: &[i64],
    dists: &[f64],
    point_map: &mut HashMap<usize, usize>,
    edge_locator: &mut HashMap<(i64, i64), i64>,
    input: &UnstructuredGrid,
    out_points: &mut Points<f64>,
    point_sources: &mut Vec<TupleSource>,
) -> Vec<i64> {
    if ids.len() != 2 {
        return Vec::new();
    }

    let d0 = dists[ids[0] as usize];
    let d1 = dists[ids[1] as usize];
    let in0 = d0 > 0.0;
    let in1 = d1 > 0.0;
    if in0 && in1 {
        return remap_existing_points(ids, point_map, &input.points, out_points, point_sources);
    }
    if !in0 && !in1 {
        return Vec::new();
    }

    let x = get_or_insert_intersection(
        ids[0],
        ids[1],
        d0,
        d1,
        input,
        out_points,
        edge_locator,
        point_sources,
    );
    if in0 {
        let p0 = remap_existing_points(
            &[ids[0]],
            point_map,
            &input.points,
            out_points,
            point_sources,
        )[0];
        vec![p0, x]
    } else {
        let p1 = remap_existing_points(
            &[ids[1]],
            point_map,
            &input.points,
            out_points,
            point_sources,
        )[0];
        vec![x, p1]
    }
}

fn copy_point_data(
    input: &UnstructuredGrid,
    sources: &[TupleSource],
    output: &mut UnstructuredGrid,
) {
    copy_attributes(input.point_data(), sources, output.point_data_mut());
}

fn copy_cell_data(input: &UnstructuredGrid, source_ids: &[usize], output: &mut UnstructuredGrid) {
    let sources: Vec<TupleSource> = source_ids.iter().copied().map(TupleSource::Copy).collect();
    copy_attributes(input.cell_data(), &sources, output.cell_data_mut());
}

fn copy_attributes(
    input: &DataSetAttributes,
    sources: &[TupleSource],
    output: &mut DataSetAttributes,
) {
    let active_scalars = input.scalars().map(|a| a.name().to_string());
    let active_vectors = input.vectors().map(|a| a.name().to_string());
    let active_normals = input.normals().map(|a| a.name().to_string());
    let active_tcoords = input.tcoords().map(|a| a.name().to_string());
    let active_tensors = input.tensors().map(|a| a.name().to_string());
    let active_global_ids = input.global_ids().map(|a| a.name().to_string());
    let active_pedigree_ids = input.pedigree_ids().map(|a| a.name().to_string());
    let active_edge_flags = input.edge_flags().map(|a| a.name().to_string());
    let active_tangents = input.tangents().map(|a| a.name().to_string());
    let active_rational_weights = input.rational_weights().map(|a| a.name().to_string());
    let active_higher_order_degrees = input.higher_order_degrees().map(|a| a.name().to_string());
    let active_process_ids = input.process_ids().map(|a| a.name().to_string());

    output.clear();
    for array in input.iter() {
        if array.num_tuples() > sources.iter().map(source_max_id).max().unwrap_or(0) {
            output.add_array(map_array(array, sources));
        }
    }

    if let Some(name) = active_scalars.as_deref() {
        output.set_active_scalars(name);
    }
    if let Some(name) = active_vectors.as_deref() {
        output.set_active_vectors(name);
    }
    if let Some(name) = active_normals.as_deref() {
        output.set_active_normals(name);
    }
    if let Some(name) = active_tcoords.as_deref() {
        output.set_active_tcoords(name);
    }
    if let Some(name) = active_tensors.as_deref() {
        output.set_active_tensors(name);
    }
    if let Some(name) = active_global_ids.as_deref() {
        output.set_active_global_ids(name);
    }
    if let Some(name) = active_pedigree_ids.as_deref() {
        output.set_active_pedigree_ids(name);
    }
    if let Some(name) = active_edge_flags.as_deref() {
        output.set_active_edge_flags(name);
    }
    if let Some(name) = active_tangents.as_deref() {
        output.set_active_tangents(name);
    }
    if let Some(name) = active_rational_weights.as_deref() {
        output.set_active_rational_weights(name);
    }
    if let Some(name) = active_higher_order_degrees.as_deref() {
        output.set_active_higher_order_degrees(name);
    }
    if let Some(name) = active_process_ids.as_deref() {
        output.set_active_process_ids(name);
    }
}

fn source_max_id(source: &TupleSource) -> usize {
    match *source {
        TupleSource::Copy(id) => id,
        TupleSource::Interpolate { a, b, .. } => a.max(b),
    }
}

fn map_array(array: &AnyDataArray, sources: &[TupleSource]) -> AnyDataArray {
    macro_rules! map {
        ($arr:expr, $variant:ident) => {
            AnyDataArray::$variant(map_typed_array($arr, sources))
        };
    }
    match array {
        AnyDataArray::F32(a) => map!(a, F32),
        AnyDataArray::F64(a) => map!(a, F64),
        AnyDataArray::I8(a) => map!(a, I8),
        AnyDataArray::I16(a) => map!(a, I16),
        AnyDataArray::I32(a) => map!(a, I32),
        AnyDataArray::I64(a) => map!(a, I64),
        AnyDataArray::U8(a) => map!(a, U8),
        AnyDataArray::U16(a) => map!(a, U16),
        AnyDataArray::U32(a) => map!(a, U32),
        AnyDataArray::U64(a) => map!(a, U64),
    }
}

fn map_typed_array<T: Scalar>(array: &DataArray<T>, sources: &[TupleSource]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(sources.len() * num_components);
    for source in sources {
        match *source {
            TupleSource::Copy(id) => data.extend_from_slice(array.tuple(id)),
            TupleSource::Interpolate { a, b, t } => {
                let ta = array.tuple(a);
                let tb = array.tuple(b);
                for component in 0..num_components {
                    let value = ta[component].to_f64() * (1.0 - t) + tb[component].to_f64() * t;
                    data.push(T::from_f64(value));
                }
            }
        }
    }
    DataArray::from_vec(array.name(), data, num_components)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CellType;

    #[test]
    fn clip_keeps_inside_cells() {
        let mut grid = UnstructuredGrid::new();
        // Two triangles: one at x>0, one at x<0
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([2.0, 0.0, 0.0]);
        grid.points.push([1.5, 1.0, 0.0]);
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([-2.0, 0.0, 0.0]);
        grid.points.push([-1.5, 1.0, 0.0]);

        grid.push_cell(CellType::Triangle, &[0, 1, 2]);
        grid.push_cell(CellType::Triangle, &[3, 4, 5]);

        // Clip at x=0, keep x>0
        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn clip_removes_all() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([-2.0, 0.0, 0.0]);
        grid.points.push([-1.5, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 0);
    }

    #[test]
    fn clip_mixed_cells() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        // All vertices have positive distance, so the tetra should be kept.
        let result = clip_data_set(&grid, [-0.1, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 1);
    }

    #[test]
    fn clip_splits_triangle_cell() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(result.cells().num_cells() > 0);
        for i in 0..result.points.len() {
            assert!(result.points.get(i)[0] >= -1e-12);
        }
    }

    #[test]
    fn clip_tetra_with_one_inside_vertex() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([-1.0, 1.0, 0.0]);
        grid.points.push([-1.0, 0.0, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Tetra);
        for i in 0..result.points.len() {
            assert!(result.points.get(i)[0] >= -1e-12);
        }
    }

    #[test]
    fn clip_tetra_with_two_inside_vertices() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([1.0, 1.0, 0.0]);
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([-1.0, 0.0, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Wedge);
        for i in 0..result.points.len() {
            assert!(result.points.get(i)[0] >= -1e-12);
        }
    }

    #[test]
    fn clip_tetra_with_three_inside_vertices() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([1.0, 1.0, 0.0]);
        grid.points.push([1.0, 0.0, 1.0]);
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.cells().num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Wedge);
        for i in 0..result.points.len() {
            assert!(result.points.get(i)[0] >= -1e-12);
        }
    }

    #[test]
    fn clip_interpolates_point_data_and_copies_cell_data() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([-1.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_values",
                vec![10.0, 30.0, 20.0],
                1,
            )));
        grid.point_data_mut().set_active_scalars("point_values");
        grid.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_ids",
                vec![7],
                1,
            )));

        let result = clip_data_set(&grid, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);

        let point_values = result.point_data().get_array("point_values").unwrap();
        assert_eq!(point_values.num_tuples(), result.points.len());
        assert!(result.point_data().scalars().is_some());
        let mut value = [0.0];
        point_values.tuple_as_f64(0, &mut value);
        assert!((value[0] - 20.0).abs() < 1e-12);

        let cell_ids = result.cell_data().get_array("cell_ids").unwrap();
        assert_eq!(cell_ids.num_tuples(), result.cells().num_cells());
        cell_ids.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 7.0);
    }
}
