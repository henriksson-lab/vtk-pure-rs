//! HedgeHog filter: oriented line glyphs from vector fields.
//!
//! For each point with a vector data array, creates a line segment
//! from the point in the direction of the vector, scaled by a factor.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

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

    // VTK orders all original points first, then all displaced endpoints.
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

        let base_out = i * 3;
        let tip_out = (i + n) * 3;
        pts_flat[base_out] = px;
        pts_flat[base_out + 1] = py;
        pts_flat[base_out + 2] = pz;
        pts_flat[tip_out] = px + vbuf[0] * scale_factor;
        pts_flat[tip_out + 1] = py + vbuf[1] * scale_factor;
        pts_flat[tip_out + 2] = pz + vbuf[2] * scale_factor;

        let base_idx = i as i64;
        conn.push(base_idx);
        conn.push(base_idx + n as i64);
        offsets.push((i as i64 + 1) * 2);
    }

    let mut result = PolyData::new();
    result.points = Points::from_flat_vec(pts_flat);
    result.lines = CellArray::from_raw(offsets, conn);
    copy_duplicated_point_data(input, &mut result, n);
    result
}

fn copy_duplicated_point_data(input: &PolyData, output: &mut PolyData, n: usize) {
    for arr in input.point_data().iter() {
        if arr.num_tuples() < n {
            continue;
        }
        output
            .point_data_mut()
            .add_array(duplicate_first_n_tuples(arr, n));
    }

    let active_scalars = input.point_data().scalars().map(|a| a.name().to_string());
    let active_vectors = input.point_data().vectors().map(|a| a.name().to_string());
    let active_normals = input.point_data().normals().map(|a| a.name().to_string());
    let active_tcoords = input.point_data().tcoords().map(|a| a.name().to_string());
    let active_tensors = input.point_data().tensors().map(|a| a.name().to_string());

    if let Some(name) = active_scalars {
        output.point_data_mut().set_active_scalars(&name);
    }
    if let Some(name) = active_vectors {
        output.point_data_mut().set_active_vectors(&name);
    }
    if let Some(name) = active_normals {
        output.point_data_mut().set_active_normals(&name);
    }
    if let Some(name) = active_tcoords {
        output.point_data_mut().set_active_tcoords(&name);
    }
    if let Some(name) = active_tensors {
        output.point_data_mut().set_active_tensors(&name);
    }
}

fn duplicate_first_n_tuples(arr: &AnyDataArray, n: usize) -> AnyDataArray {
    macro_rules! duplicate {
        ($array:expr, $variant:ident) => {{
            let nc = $array.num_components();
            let mut data = Vec::with_capacity(n * nc * 2);
            for i in 0..n {
                data.extend_from_slice($array.tuple(i));
            }
            for i in 0..n {
                data.extend_from_slice($array.tuple(i));
            }
            AnyDataArray::$variant(DataArray::from_vec($array.name(), data, nc))
        }};
    }

    match arr {
        AnyDataArray::F32(a) => duplicate!(a, F32),
        AnyDataArray::F64(a) => duplicate!(a, F64),
        AnyDataArray::I8(a) => duplicate!(a, I8),
        AnyDataArray::I16(a) => duplicate!(a, I16),
        AnyDataArray::I32(a) => duplicate!(a, I32),
        AnyDataArray::I64(a) => duplicate!(a, I64),
        AnyDataArray::U8(a) => duplicate!(a, U8),
        AnyDataArray::U16(a) => duplicate!(a, U16),
        AnyDataArray::U32(a) => duplicate!(a, U32),
        AnyDataArray::U64(a) => duplicate!(a, U64),
    }
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
        let tip = result.points.get(3);
        assert!((tip[0] - 2.0).abs() < 1e-10);
        assert_eq!(
            result
                .point_data()
                .get_array("vectors")
                .unwrap()
                .num_tuples(),
            6
        );
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
