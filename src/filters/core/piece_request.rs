//! Streaming/chunked data loading for large datasets.
//!
//! Provides utilities for processing data in pieces (chunks) without
//! loading the entire dataset into memory at once.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData, Points, PolyData};

/// A piece (chunk) specification for streaming.
#[derive(Debug, Clone)]
pub struct PieceExtent {
    /// Start indices [x, y, z].
    pub start: [usize; 3],
    /// End indices (exclusive) [x, y, z].
    pub end: [usize; 3],
}

/// Split an ImageData extent into N pieces along the longest axis.
pub fn compute_pieces(dims: [usize; 3], n_pieces: usize) -> Vec<PieceExtent> {
    if n_pieces == 0 || dims.contains(&0) {
        return Vec::new();
    }

    let longest = if dims[0] >= dims[1] && dims[0] >= dims[2] {
        0
    } else if dims[1] >= dims[2] {
        1
    } else {
        2
    };

    let total = dims[longest];
    let mut pieces = Vec::with_capacity(n_pieces);
    let chunk_size = (total + n_pieces - 1) / n_pieces;

    for i in 0..n_pieces {
        let start_idx = i * chunk_size;
        let end_idx = ((i + 1) * chunk_size).min(total);
        if start_idx >= total {
            break;
        }

        let mut start = [0; 3];
        let mut end = dims;
        start[longest] = start_idx;
        end[longest] = end_idx;
        pieces.push(PieceExtent { start, end });
    }
    pieces
}

/// Extract a piece (sub-region) from an ImageData.
pub fn extract_piece(image: &ImageData, piece: &PieceExtent) -> ImageData {
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();

    let start = [
        piece.start[0].min(dims[0]),
        piece.start[1].min(dims[1]),
        piece.start[2].min(dims[2]),
    ];
    let end = [
        piece.end[0].min(dims[0]).max(start[0]),
        piece.end[1].min(dims[1]).max(start[1]),
        piece.end[2].min(dims[2]).max(start[2]),
    ];
    let new_dims = [end[0] - start[0], end[1] - start[1], end[2] - start[2]];
    let new_origin = [
        origin[0] + start[0] as f64 * spacing[0],
        origin[1] + start[1] as f64 * spacing[1],
        origin[2] + start[2] as f64 * spacing[2],
    ];

    let mut result = ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(spacing)
        .with_origin(new_origin);

    // Extract point data
    let pd = image.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let name = arr.name().to_string();
            if let Some(subset) = extract_array_piece(arr, dims, start, end) {
                result.point_data_mut().add_array(subset);
            }
            copy_active_attribute_for_array(pd, result.point_data_mut(), &name);
        }
    }

    result
}

/// Split a PolyData point cloud into N pieces by point index.
pub fn split_points_into_pieces(mesh: &PolyData, n_pieces: usize) -> Vec<PolyData> {
    let n = mesh.points.len();
    if n == 0 || n_pieces == 0 {
        return Vec::new();
    }
    let chunk = (n + n_pieces - 1) / n_pieces;

    let mut pieces = Vec::new();
    for i in 0..n_pieces {
        let start = i * chunk;
        let end = ((i + 1) * chunk).min(n);
        if start >= n {
            break;
        }

        let mut pts = Points::<f64>::new();
        for j in start..end {
            pts.push(mesh.points.get(j));
        }
        let mut piece = PolyData::new();
        piece.points = pts;
        copy_point_data_range(mesh.point_data(), piece.point_data_mut(), start, end);
        pieces.push(piece);
    }
    pieces
}

fn copy_point_data_range(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    start: usize,
    end: usize,
) {
    for array in source.iter() {
        if end > array.num_tuples() {
            continue;
        }
        if let Some(subset) = slice_array(array, start, end) {
            let name = subset.name().to_string();
            target.add_array(subset);
            copy_active_attribute_for_array(source, target, &name);
        }
    }
}

fn copy_active_attribute_for_array(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
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

fn slice_array(array: &AnyDataArray, start: usize, end: usize) -> Option<AnyDataArray> {
    macro_rules! slice_variant {
        ($variant:ident, $a:expr) => {{
            let nc = $a.num_components();
            let from = start.checked_mul(nc)?;
            let to = end.checked_mul(nc)?;
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $a.name(),
                $a.as_slice().get(from..to)?.to_vec(),
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(a) => slice_variant!(F32, a),
        AnyDataArray::F64(a) => slice_variant!(F64, a),
        AnyDataArray::I8(a) => slice_variant!(I8, a),
        AnyDataArray::I16(a) => slice_variant!(I16, a),
        AnyDataArray::I32(a) => slice_variant!(I32, a),
        AnyDataArray::I64(a) => slice_variant!(I64, a),
        AnyDataArray::U8(a) => slice_variant!(U8, a),
        AnyDataArray::U16(a) => slice_variant!(U16, a),
        AnyDataArray::U32(a) => slice_variant!(U32, a),
        AnyDataArray::U64(a) => slice_variant!(U64, a),
    }
}

fn extract_array_piece(
    array: &AnyDataArray,
    dims: [usize; 3],
    start: [usize; 3],
    end: [usize; 3],
) -> Option<AnyDataArray> {
    macro_rules! extract_variant {
        ($variant:ident, $a:expr) => {{
            let nc = $a.num_components();
            let nt = (end[0] - start[0]) * (end[1] - start[1]) * (end[2] - start[2]);
            let mut data = Vec::with_capacity(nt * nc);
            for iz in start[2]..end[2] {
                for iy in start[1]..end[1] {
                    for ix in start[0]..end[0] {
                        let old_idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                        let from = old_idx.checked_mul(nc)?;
                        let to = from.checked_add(nc)?;
                        data.extend_from_slice($a.as_slice().get(from..to)?);
                    }
                }
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $a.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(a) => extract_variant!(F32, a),
        AnyDataArray::F64(a) => extract_variant!(F64, a),
        AnyDataArray::I8(a) => extract_variant!(I8, a),
        AnyDataArray::I16(a) => extract_variant!(I16, a),
        AnyDataArray::I32(a) => extract_variant!(I32, a),
        AnyDataArray::I64(a) => extract_variant!(I64, a),
        AnyDataArray::U8(a) => extract_variant!(U8, a),
        AnyDataArray::U16(a) => extract_variant!(U16, a),
        AnyDataArray::U32(a) => extract_variant!(U32, a),
        AnyDataArray::U64(a) => extract_variant!(U64, a),
    }
}

/// Process an ImageData in streaming fashion, applying a function to each piece.
pub fn stream_process_image<F>(image: &ImageData, n_pieces: usize, process: F) -> Vec<ImageData>
where
    F: Fn(&ImageData, usize) -> ImageData,
{
    let pieces = compute_pieces(image.dimensions(), n_pieces);
    pieces
        .iter()
        .enumerate()
        .map(|(i, piece)| {
            let sub = extract_piece(image, piece);
            process(&sub, i)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_4_pieces() {
        let pieces = compute_pieces([20, 10, 10], 4);
        assert_eq!(pieces.len(), 4);
        assert_eq!(pieces[0].start[0], 0);
        assert_eq!(pieces[0].end[0], 5);
        assert_eq!(pieces[3].end[0], 20);
    }

    #[test]
    fn compute_zero_pieces_returns_empty() {
        assert!(compute_pieces([20, 10, 10], 0).is_empty());
        assert!(compute_pieces([0, 10, 10], 4).is_empty());
    }

    #[test]
    fn extract_piece_test() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _y, _z| x,
        );
        let piece = PieceExtent {
            start: [2, 0, 0],
            end: [5, 10, 1],
        };
        let sub = extract_piece(&img, &piece);
        assert_eq!(sub.dimensions(), [3, 10, 1]);
        assert!(sub.point_data().scalars().is_some());
        assert_eq!(sub.scalar_at(0, 0, 0), Some(2.0));
    }

    #[test]
    fn extract_piece_preserves_array_type_and_active_vectors() {
        let mut img = ImageData::with_dimensions(3, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "id",
                vec![0, 1, 2, 3, 4, 5],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F32(DataArray::from_vec(
                "v",
                vec![
                    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 3.0, 0.0, 0.0, 4.0, 0.0, 0.0, 5.0,
                    0.0, 0.0,
                ],
                3,
            )));
        img.point_data_mut().set_active_scalars("id");
        img.point_data_mut().set_active_vectors("v");

        let sub = extract_piece(
            &img,
            &PieceExtent {
                start: [1, 0, 0],
                end: [3, 2, 1],
            },
        );

        assert!(matches!(
            sub.point_data().scalars().unwrap(),
            AnyDataArray::U8(_)
        ));
        assert!(matches!(
            sub.point_data().vectors().unwrap(),
            AnyDataArray::F32(_)
        ));
        assert_eq!(
            sub.point_data().scalars().unwrap().to_f64_vec(),
            vec![1.0, 2.0, 4.0, 5.0]
        );
    }

    #[test]
    fn split_points() {
        let mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
        ]);
        let pieces = split_points_into_pieces(&mesh, 2);
        assert_eq!(pieces.len(), 2);
        assert_eq!(pieces[0].points.len(), 3);
        assert_eq!(pieces[1].points.len(), 2);
    }

    #[test]
    fn split_points_preserves_point_data_ranges() {
        let mut mesh = PolyData::from_points((0..5).map(|i| [i as f64, 0.0, 0.0]).collect());
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 2.0, 3.0, 4.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("s");

        let pieces = split_points_into_pieces(&mesh, 2);

        assert_eq!(
            pieces[1].point_data().scalars().unwrap().to_f64_vec(),
            vec![3.0, 4.0]
        );
    }

    #[test]
    fn split_and_extract_preserve_active_tcoords() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F32(DataArray::from_vec(
                "tc",
                vec![0.0, 0.0, 0.5, 0.0, 1.0, 0.0],
                2,
            )));
        img.point_data_mut().set_active_tcoords("tc");

        let sub = extract_piece(
            &img,
            &PieceExtent {
                start: [1, 0, 0],
                end: [3, 1, 1],
            },
        );
        assert!(matches!(
            sub.point_data().tcoords().unwrap(),
            AnyDataArray::F32(_)
        ));

        let mut mesh = PolyData::from_points((0..3).map(|i| [i as f64, 0.0, 0.0]).collect());
        mesh.point_data_mut()
            .add_array(AnyDataArray::F32(DataArray::from_vec(
                "tc",
                vec![0.0, 0.0, 0.5, 0.0, 1.0, 0.0],
                2,
            )));
        mesh.point_data_mut().set_active_tcoords("tc");

        let pieces = split_points_into_pieces(&mesh, 2);
        assert!(matches!(
            pieces[0].point_data().tcoords().unwrap(),
            AnyDataArray::F32(_)
        ));
    }

    #[test]
    fn stream_process() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _y, _z| x,
        );
        let results = stream_process_image(&img, 2, |sub, _| sub.clone());
        assert_eq!(results.len(), 2);
    }
}
