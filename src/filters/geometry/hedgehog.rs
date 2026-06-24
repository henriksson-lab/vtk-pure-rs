//! HedgeHog filter: oriented line glyphs from vector fields.
//!
//! For each point with a vector data array, creates a line segment
//! from the point in the direction of the vector, scaled by a factor.

use crate::data::{CellArray, Points, PolyData};

/// Generate hedgehog (oriented line) glyphs from a vector field.
///
/// Each point produces a line from the point position to
/// `position + vector * scale_factor`.
pub fn hedgehog(input: &PolyData, vector_name: &str, scale_factor: f64) -> PolyData {
    let vectors = match input.point_data().get_array(vector_name) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };

    let n = input.points.len();
    let nc = vectors.num_components();
    if nc < 3 {
        return PolyData::new();
    }

    // Pre-allocated flat buffers: 2 points per input point (base + tip), 3 coords each.
    // Sequential connectivity avoids per-cell push_cell() overhead.
    let pts_in = input.points.as_flat_slice();
    let mut pts_flat = vec![0.0f64; n * 6];
    let mut offsets = Vec::with_capacity(n + 1);
    let mut conn = Vec::with_capacity(n * 2);
    offsets.push(0i64);

    let mut vbuf = [0.0f64; 3];
    for i in 0..n {
        let b = i * 3;
        let px = pts_in[b];
        let py = pts_in[b + 1];
        let pz = pts_in[b + 2];
        vectors.tuple_as_f64(i, &mut vbuf);

        let out = i * 6;
        pts_flat[out] = px;
        pts_flat[out + 1] = py;
        pts_flat[out + 2] = pz;
        pts_flat[out + 3] = px + vbuf[0] * scale_factor;
        pts_flat[out + 4] = py + vbuf[1] * scale_factor;
        pts_flat[out + 5] = pz + vbuf[2] * scale_factor;

        let base_idx = (i * 2) as i64;
        conn.push(base_idx);
        conn.push(base_idx + 1);
        offsets.push((i as i64 + 1) * 2);
    }

    let mut result = PolyData::new();
    result.points = Points::from_flat_vec(pts_flat);
    result.lines = CellArray::from_raw(offsets, conn);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn hedgehog_basic() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "vectors",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                3,
            )));
        let result = hedgehog(&pd, "vectors", 2.0);
        assert_eq!(result.points.len(), 6); // 3 points × 2 (base + tip)
        assert_eq!(result.lines.num_cells(), 3);
        // Check first tip
        let tip = result.points.get(1);
        assert!((tip[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn hedgehog_missing_array() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = hedgehog(&pd, "nonexistent", 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
