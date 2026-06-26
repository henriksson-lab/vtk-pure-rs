//! Streaming/chunked data loading for large datasets.
//!
//! Provides utilities for processing data in pieces (chunks) without
//! loading the entire dataset into memory at once.

use crate::data::{AnyDataArray, DataArray, ImageData, Points, PolyData};

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
    if n_pieces == 0 {
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

    let new_dims = [
        piece.end[0] - piece.start[0],
        piece.end[1] - piece.start[1],
        piece.end[2] - piece.start[2],
    ];
    let new_origin = [
        origin[0] + piece.start[0] as f64 * spacing[0],
        origin[1] + piece.start[1] as f64 * spacing[1],
        origin[2] + piece.start[2] as f64 * spacing[2],
    ];

    let mut result = ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(spacing)
        .with_origin(new_origin);

    // Extract point data
    let pd = image.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let nc = arr.num_components();
            let name = arr.name().to_string();
            let mut data = Vec::new();
            let mut buf = vec![0.0f64; nc];

            for iz in piece.start[2]..piece.end[2] {
                for iy in piece.start[1]..piece.end[1] {
                    for ix in piece.start[0]..piece.end[0] {
                        let old_idx = ix + iy * dims[0] + iz * dims[0] * dims[1];
                        if old_idx < arr.num_tuples() {
                            arr.tuple_as_f64(old_idx, &mut buf);
                            data.extend_from_slice(&buf);
                        }
                    }
                }
            }

            result
                .point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, nc)));
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
        pieces.push(piece);
    }
    pieces
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
    fn compute_zero_pieces() {
        let pieces = compute_pieces([20, 10, 10], 0);
        assert!(pieces.is_empty());
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
