//! Spatial data decomposition for distributed processing.
//!
//! Partitions meshes and grids into pieces for MPI distribution.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData, PolyData};

/// A partition of a distributed dataset.
#[derive(Debug, Clone)]
pub struct Partition {
    /// Rank/process ID that owns this partition.
    pub rank: usize,
    /// The local data for this partition.
    pub data: PolyData,
    /// Global point IDs (maps local index → global index).
    pub global_point_ids: Vec<usize>,
    /// Global cell IDs.
    pub global_cell_ids: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Verts,
    Lines,
    Polys,
    Strips,
}

#[derive(Debug, Clone, Copy)]
struct CellRef {
    flat_id: usize,
    kind: CellKind,
    local_id: usize,
    centroid: [f64; 3],
}

/// Decompose a PolyData into N partitions using spatial bisection.
///
/// Recursively bisects along the longest axis of the bounding box.
/// Each partition gets roughly equal numbers of cells.
pub fn decompose_poly_data(input: &PolyData, num_partitions: usize) -> Vec<Partition> {
    if num_partitions == 0 {
        return Vec::new();
    }
    if num_partitions == 1 || input.total_cells() == 0 {
        return vec![Partition {
            rank: 0,
            data: input.clone(),
            global_point_ids: (0..input.points.len()).collect(),
            global_cell_ids: (0..input.total_cells()).collect(),
        }];
    }

    // Compute cell centroids
    let nc = input.total_cells();
    let mut centroids: Vec<CellRef> = Vec::with_capacity(nc);
    collect_centroids(input, CellKind::Verts, &input.verts, &mut centroids);
    collect_centroids(input, CellKind::Lines, &input.lines, &mut centroids);
    collect_centroids(input, CellKind::Polys, &input.polys, &mut centroids);
    collect_centroids(input, CellKind::Strips, &input.strips, &mut centroids);

    let cell_groups = recursive_cell_split(centroids, num_partitions);
    let partitions = cell_groups
        .iter()
        .enumerate()
        .map(|(rank, chunk)| extract_partition(input, rank, chunk))
        .collect();

    partitions
}

/// Decompose ImageData into N pieces using VTK-style block extent splitting.
pub fn decompose_image_data(input: &ImageData, num_partitions: usize) -> Vec<ImageData> {
    let dims = input.dimensions();
    if num_partitions == 0 {
        return Vec::new();
    }
    if num_partitions == 1 {
        return vec![input.clone()];
    }

    let spacing = input.spacing();
    let mut parts = Vec::with_capacity(num_partitions);

    for p in 0..num_partitions {
        let Some(extent) = split_extent_by_points(p, num_partitions, input.extent()) else {
            parts.push(empty_image_like(input));
            continue;
        };
        let local_dims = extent_dimensions(extent);

        let mut slab = ImageData::with_dimensions(local_dims[0], local_dims[1], local_dims[2]);
        slab.set_spacing(spacing);
        slab.set_origin([
            input.origin()[0] + extent[0] as f64 * spacing[0],
            input.origin()[1] + extent[2] as f64 * spacing[1],
            input.origin()[2] + extent[4] as f64 * spacing[2],
        ]);

        let mut tuple_ids = Vec::with_capacity(local_dims[0] * local_dims[1] * local_dims[2]);
        for k in extent[4]..=extent[5] {
            for j in extent[2]..=extent[3] {
                for i in extent[0]..=extent[1] {
                    tuple_ids.push(point_tuple_id(input.extent(), dims, i, j, k));
                }
            }
        }
        copy_selected_attributes(input.point_data(), slab.point_data_mut(), &tuple_ids);

        let cell_tuple_ids = cell_tuple_ids_for_extent(input.extent(), dims, extent);
        copy_selected_attributes(input.cell_data(), slab.cell_data_mut(), &cell_tuple_ids);

        parts.push(slab);
    }

    parts
}

fn recursive_cell_split(mut cells: Vec<CellRef>, num_partitions: usize) -> Vec<Vec<CellRef>> {
    if num_partitions == 0 {
        return Vec::new();
    }
    if num_partitions == 1 || cells.len() <= 1 {
        let mut groups = Vec::with_capacity(num_partitions);
        groups.push(cells);
        groups.resize_with(num_partitions, Vec::new);
        return groups;
    }

    let axis = longest_centroid_axis(&cells);
    cells.sort_by(|a, b| {
        a.centroid[axis]
            .partial_cmp(&b.centroid[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let num_first_half = num_partitions / 2;
    let split = cells.len() * num_first_half / num_partitions;
    let right = cells.split_off(split);

    let mut groups = recursive_cell_split(cells, num_first_half);
    groups.extend(recursive_cell_split(right, num_partitions - num_first_half));
    groups
}

fn longest_centroid_axis(cells: &[CellRef]) -> usize {
    let (mut min, mut max) = ([f64::MAX; 3], [f64::MIN; 3]);
    for cell_ref in cells {
        for axis in 0..3 {
            min[axis] = min[axis].min(cell_ref.centroid[axis]);
            max[axis] = max[axis].max(cell_ref.centroid[axis]);
        }
    }
    let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    if size[2] >= size[1] && size[2] >= size[0] {
        2
    } else if size[1] >= size[0] {
        1
    } else {
        0
    }
}

fn split_extent_by_points(
    piece: usize,
    num_pieces: usize,
    mut extent: [i64; 6],
) -> Option<[i64; 6]> {
    if piece >= num_pieces {
        return None;
    }

    let mut piece = piece;
    let mut num_pieces = num_pieces;
    while num_pieces > 1 {
        let size = extent_dimensions_i64(extent);
        let split_axis = if size[2] >= size[1] && size[2] >= size[0] && size[2] / 2 >= 1 {
            Some(2)
        } else if size[1] >= size[0] && size[1] / 2 >= 1 {
            Some(1)
        } else if size[0] / 2 >= 1 {
            Some(0)
        } else {
            None
        };

        let Some(split_axis) = split_axis else {
            return (piece == 0).then_some(extent);
        };

        let num_pieces_in_first_half = num_pieces / 2;
        let mid = size[split_axis] * num_pieces_in_first_half as i64 / num_pieces as i64
            + extent[split_axis * 2];
        if piece < num_pieces_in_first_half {
            extent[split_axis * 2 + 1] = mid - 1;
            num_pieces = num_pieces_in_first_half;
        } else {
            extent[split_axis * 2] = mid;
            num_pieces -= num_pieces_in_first_half;
            piece -= num_pieces_in_first_half;
        }
    }

    Some(extent)
}

fn empty_image_like(input: &ImageData) -> ImageData {
    let mut image = ImageData::with_dimensions(0, 0, 0);
    image.set_spacing(input.spacing());
    image.set_origin(input.origin());
    image
}

fn extent_dimensions_i64(extent: [i64; 6]) -> [i64; 3] {
    [
        extent[1] - extent[0] + 1,
        extent[3] - extent[2] + 1,
        extent[5] - extent[4] + 1,
    ]
}

fn extent_dimensions(extent: [i64; 6]) -> [usize; 3] {
    let dims = extent_dimensions_i64(extent);
    [
        dims[0].max(0) as usize,
        dims[1].max(0) as usize,
        dims[2].max(0) as usize,
    ]
}

fn point_tuple_id(extent: [i64; 6], dims: [usize; 3], i: i64, j: i64, k: i64) -> usize {
    let ii = (i - extent[0]) as usize;
    let jj = (j - extent[2]) as usize;
    let kk = (k - extent[4]) as usize;
    kk * dims[0] * dims[1] + jj * dims[0] + ii
}

fn cell_tuple_ids_for_extent(
    whole_extent: [i64; 6],
    whole_dims: [usize; 3],
    piece_extent: [i64; 6],
) -> Vec<usize> {
    let whole_cell_dims = cell_dimensions(whole_dims);
    if whole_cell_dims.contains(&0) {
        return Vec::new();
    }

    let piece_dims = extent_dimensions(piece_extent);
    let piece_cell_dims = cell_dimensions(piece_dims);
    if piece_cell_dims.contains(&0) {
        return Vec::new();
    }

    let i_end = piece_extent[0] + piece_cell_dims[0] as i64 - 1;
    let j_end = piece_extent[2] + piece_cell_dims[1] as i64 - 1;
    let k_end = piece_extent[4] + piece_cell_dims[2] as i64 - 1;

    let mut tuple_ids =
        Vec::with_capacity(piece_cell_dims[0] * piece_cell_dims[1] * piece_cell_dims[2]);
    for k in piece_extent[4]..=k_end {
        for j in piece_extent[2]..=j_end {
            for i in piece_extent[0]..=i_end {
                let ii = (i - whole_extent[0]) as usize;
                let jj = (j - whole_extent[2]) as usize;
                let kk = (k - whole_extent[4]) as usize;
                tuple_ids.push(
                    kk * whole_cell_dims[0] * whole_cell_dims[1] + jj * whole_cell_dims[0] + ii,
                );
            }
        }
    }
    tuple_ids
}

fn cell_dimensions(point_dims: [usize; 3]) -> [usize; 3] {
    if point_dims.contains(&0) {
        return [0, 0, 0];
    }
    [
        point_dims[0].saturating_sub(1).max(1),
        point_dims[1].saturating_sub(1).max(1),
        point_dims[2].saturating_sub(1).max(1),
    ]
}

fn collect_centroids(
    input: &PolyData,
    kind: CellKind,
    cells: &crate::data::CellArray,
    output: &mut Vec<CellRef>,
) {
    for local_id in 0..cells.num_cells() {
        let cell = cells.cell(local_id);
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &vid in cell {
            let p = input.points.get(vid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        let centroid = if n > 0.0 {
            [cx / n, cy / n, cz / n]
        } else {
            [0.0, 0.0, 0.0]
        };
        output.push(CellRef {
            flat_id: output.len(),
            kind,
            local_id,
            centroid,
        });
    }
}

fn extract_partition(input: &PolyData, rank: usize, cell_refs: &[CellRef]) -> Partition {
    let mut point_map = vec![usize::MAX; input.points.len()];
    let mut new_points = crate::data::Points::<f64>::new();
    let mut global_point_ids = Vec::new();
    let mut data = PolyData::new();
    let mut global_cell_ids = Vec::with_capacity(cell_refs.len());

    append_partition_cells(
        input,
        cell_refs,
        CellKind::Verts,
        &input.verts,
        &mut data.verts,
        &mut point_map,
        &mut new_points,
        &mut global_point_ids,
        &mut global_cell_ids,
    );
    append_partition_cells(
        input,
        cell_refs,
        CellKind::Lines,
        &input.lines,
        &mut data.lines,
        &mut point_map,
        &mut new_points,
        &mut global_point_ids,
        &mut global_cell_ids,
    );
    append_partition_cells(
        input,
        cell_refs,
        CellKind::Polys,
        &input.polys,
        &mut data.polys,
        &mut point_map,
        &mut new_points,
        &mut global_point_ids,
        &mut global_cell_ids,
    );
    append_partition_cells(
        input,
        cell_refs,
        CellKind::Strips,
        &input.strips,
        &mut data.strips,
        &mut point_map,
        &mut new_points,
        &mut global_point_ids,
        &mut global_cell_ids,
    );

    data.points = new_points;
    copy_selected_attributes(input.point_data(), data.point_data_mut(), &global_point_ids);
    copy_selected_attributes(input.cell_data(), data.cell_data_mut(), &global_cell_ids);

    Partition {
        rank,
        data,
        global_point_ids,
        global_cell_ids,
    }
}

fn append_partition_cells(
    input: &PolyData,
    cell_refs: &[CellRef],
    kind: CellKind,
    source_cells: &crate::data::CellArray,
    target_cells: &mut crate::data::CellArray,
    point_map: &mut [usize],
    new_points: &mut crate::data::Points<f64>,
    global_point_ids: &mut Vec<usize>,
    global_cell_ids: &mut Vec<usize>,
) {
    for cell_ref in cell_refs.iter().filter(|cell_ref| cell_ref.kind == kind) {
        let cell = source_cells.cell(cell_ref.local_id);
        for &vid in cell {
            let vi = vid as usize;
            if point_map[vi] == usize::MAX {
                point_map[vi] = new_points.len();
                global_point_ids.push(vi);
                new_points.push(input.points.get(vi));
            }
        }
        let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize] as i64).collect();
        target_cells.push_cell(&remapped);
        global_cell_ids.push(cell_ref.flat_id);
    }
}

fn copy_selected_attributes(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if array.num_tuples() <= tuple_ids.iter().copied().max().unwrap_or(0) {
            continue;
        }
        if let Some(sliced) = slice_array(array, tuple_ids) {
            let name = sliced.name().to_string();
            target.add_array(sliced);
            if source.scalars().map(|a| a.name()) == Some(name.as_str()) {
                target.set_active_scalars(&name);
            }
            if source.vectors().map(|a| a.name()) == Some(name.as_str()) {
                target.set_active_vectors(&name);
            }
            if source.normals().map(|a| a.name()) == Some(name.as_str()) {
                target.set_active_normals(&name);
            }
        }
    }
}

fn slice_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! slice_variant {
        ($arr:expr, $variant:ident) => {{
            let nc = $arr.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                data.extend_from_slice($arr.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $arr.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(a) => slice_variant!(a, F32),
        AnyDataArray::F64(a) => slice_variant!(a, F64),
        AnyDataArray::I8(a) => slice_variant!(a, I8),
        AnyDataArray::I16(a) => slice_variant!(a, I16),
        AnyDataArray::I32(a) => slice_variant!(a, I32),
        AnyDataArray::I64(a) => slice_variant!(a, I64),
        AnyDataArray::U8(a) => slice_variant!(a, U8),
        AnyDataArray::U16(a) => slice_variant!(a, U16),
        AnyDataArray::U32(a) => slice_variant!(a, U32),
        AnyDataArray::U64(a) => slice_variant!(a, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_into_two() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let parts = decompose_poly_data(&pd, 2);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].data.polys.num_cells(), 1);
        assert_eq!(parts[1].data.polys.num_cells(), 1);
        assert_eq!(parts[0].rank, 0);
        assert_eq!(parts[1].rank, 1);
    }

    #[test]
    fn decompose_single() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let parts = decompose_poly_data(&pd, 1);
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn decompose_returns_requested_partition_count() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let parts = decompose_poly_data(&pd, 4);
        assert_eq!(parts.len(), 4);
        assert_eq!(parts.iter().map(|p| p.data.total_cells()).sum::<usize>(), 2);
        assert_eq!(parts[0].rank, 0);
        assert_eq!(parts[3].rank, 3);
    }

    #[test]
    fn decompose_image_slabs() {
        let mut img = ImageData::with_dimensions(4, 4, 8);
        img.set_spacing([1.0, 1.0, 1.0]);
        let vals = vec![0.0f64; 4 * 4 * 8];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vals, 1)));

        let slabs = decompose_image_data(&img, 4);
        assert_eq!(slabs.len(), 4);
        for slab in &slabs {
            assert_eq!(slab.dimensions()[2], 2);
        }
    }

    #[test]
    fn decompose_image_slabs_preserves_point_arrays() {
        let mut img = ImageData::with_dimensions(2, 2, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "ids",
                (0..12).map(|v| v as u8).collect(),
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F32(DataArray::from_vec(
                "weights",
                (0..12).map(|v| v as f32).collect(),
                1,
            )));
        assert!(img.point_data_mut().set_active_scalars("ids"));

        let slabs = decompose_image_data(&img, 2);
        assert_eq!(slabs.len(), 2);

        let AnyDataArray::U8(ids) = slabs[0].point_data().get_array("ids").unwrap() else {
            panic!("ids array did not preserve its scalar type");
        };
        assert_eq!(ids.as_slice(), &[0, 1, 2, 3]);
        assert!(matches!(
            slabs[0].point_data().get_array("weights"),
            Some(AnyDataArray::F32(_))
        ));
        assert_eq!(slabs[0].point_data().scalars().unwrap().name(), "ids");
    }

    #[test]
    fn decompose_image_slabs_preserves_cell_arrays() {
        let mut img = ImageData::with_dimensions(2, 2, 3);
        img.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_ids",
                vec![10, 11],
                1,
            )));
        assert!(img.cell_data_mut().set_active_scalars("cell_ids"));

        let slabs = decompose_image_data(&img, 2);
        assert_eq!(slabs.len(), 2);

        let AnyDataArray::I32(first_ids) = slabs[0].cell_data().get_array("cell_ids").unwrap()
        else {
            panic!("cell_ids array did not preserve its scalar type");
        };
        let AnyDataArray::I32(second_ids) = slabs[1].cell_data().get_array("cell_ids").unwrap()
        else {
            panic!("cell_ids array did not preserve its scalar type");
        };
        assert_eq!(first_ids.as_slice(), &[10]);
        assert_eq!(second_ids.as_slice(), &[11]);
        assert_eq!(slabs[0].cell_data().scalars().unwrap().name(), "cell_ids");
    }

    #[test]
    fn decompose_image_uses_vtk_split_extent_by_points() {
        let img = ImageData::with_dimensions(8, 1, 1);
        let slabs = decompose_image_data(&img, 4);
        assert_eq!(slabs.len(), 4);
        for slab in &slabs {
            assert_eq!(slab.dimensions(), [2, 1, 1]);
        }
    }

    #[test]
    fn decompose_image_returns_empty_unsplittable_vtk_pieces() {
        let img = ImageData::with_dimensions(1, 1, 1);
        let slabs = decompose_image_data(&img, 4);
        assert_eq!(slabs.len(), 4);
        assert_eq!(slabs[0].dimensions(), [1, 1, 1]);
        assert_eq!(slabs[1].dimensions(), [0, 0, 0]);
        assert_eq!(slabs[2].dimensions(), [0, 0, 0]);
        assert_eq!(slabs[3].dimensions(), [0, 0, 0]);
    }

    #[test]
    fn decompose_empty_image_returns_requested_piece_count() {
        let img = ImageData::with_dimensions(0, 0, 0);
        let slabs = decompose_image_data(&img, 3);
        assert_eq!(slabs.len(), 3);
        assert!(slabs.iter().all(|slab| slab.dimensions() == [0, 0, 0]));
    }
}
