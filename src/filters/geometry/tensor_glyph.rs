use crate::data::{CellArray, Points, PolyData};

/// Place ellipsoid glyphs at each input point, scaled and oriented by a
/// 3×3 symmetric tensor stored as a 9-component array in point data.
///
/// The tensor is interpreted as a 3×3 matrix stored row-major. The glyph
/// is a unit sphere scaled by the eigenvalues and rotated by the
/// eigenvectors of the tensor (approximated by the matrix columns for
/// symmetric tensors).
pub fn tensor_glyph(input: &PolyData, tensor_name: &str, glyph: &PolyData) -> PolyData {
    let tensor_arr = match input.point_data().get_array(tensor_name) {
        Some(a) if a.num_components() == 9 => a,
        _ => return PolyData::new(),
    };

    let n = input.points.len();
    let glyph_n = glyph.points.len();

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    let mut buf = [0.0f64; 9];
    for i in 0..n {
        let center = input.points.get(i);
        tensor_arr.tuple_as_f64(i, &mut buf);

        // Extract columns as scale+direction (for symmetric tensors,
        // columns approximate eigenvectors scaled by eigenvalues)
        let col0 = [buf[0], buf[3], buf[6]];
        let col1 = [buf[1], buf[4], buf[7]];
        let col2 = [buf[2], buf[5], buf[8]];

        let base = out_points.len() as i64;

        // Transform each glyph point by the tensor matrix and translate
        for gi in 0..glyph_n {
            let gp = glyph.points.get(gi);
            out_points.push([
                center[0] + gp[0] * col0[0] + gp[1] * col1[0] + gp[2] * col2[0],
                center[1] + gp[0] * col0[1] + gp[1] * col1[1] + gp[2] * col2[1],
                center[2] + gp[0] * col0[2] + gp[1] * col1[2] + gp[2] * col2[2],
            ]);
        }

        for cell in glyph.polys.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            out_polys.push_cell(&shifted);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn identity_tensor_preserves_glyph() {
        let mut input = PolyData::new();
        input.points.push([0.0, 0.0, 0.0]);
        // Identity tensor (row-major)
        let tensor = DataArray::from_vec(
            "tensor",
            vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            9,
        );
        input.point_data_mut().add_array(tensor.into());

        // Simple triangle glyph
        let glyph = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );

        let result = tensor_glyph(&input, "tensor", &glyph);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        // Points should be same as glyph (identity + origin at 0)
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scaled_tensor() {
        let mut input = PolyData::new();
        input.points.push([5.0, 0.0, 0.0]);
        // Scale 2x in x, 1x in y, 0.5x in z
        let tensor = DataArray::from_vec("T", vec![2.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.5], 9);
        input.point_data_mut().add_array(tensor.into());

        let glyph = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );

        let result = tensor_glyph(&input, "T", &glyph);
        // First glyph point (1,0,0) * tensor + (5,0,0) = (7, 0, 0)
        let p0 = result.points.get(0);
        assert!((p0[0] - 7.0).abs() < 1e-10);
    }

    #[test]
    fn missing_tensor_returns_empty() {
        let input = PolyData::new();
        let glyph = PolyData::new();
        let result = tensor_glyph(&input, "nope", &glyph);
        assert_eq!(result.points.len(), 0);
    }
}
