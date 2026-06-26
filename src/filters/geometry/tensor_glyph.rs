use crate::data::{CellArray, Points, PolyData};

/// Place ellipsoid glyphs at each input point, scaled and oriented by a
/// 3×3 symmetric tensor stored as a 9-component array in point data.
///
/// The tensor is interpreted as a 3×3 matrix stored row-major. Like VTK's
/// default tensor glyph path, the symmetric part of the tensor is decomposed
/// into eigenvalues and eigenvectors; the source glyph's local axes are scaled
/// by the eigenvalues and rotated onto the corresponding eigenvectors.
pub fn tensor_glyph(input: &PolyData, tensor_name: &str, glyph: &PolyData) -> PolyData {
    let tensor_arr = match input.point_data().get_array(tensor_name) {
        Some(a) if a.num_components() == 6 || a.num_components() == 9 => a,
        _ => return PolyData::new(),
    };

    let n = input.points.len();
    let glyph_n = glyph.points.len();

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    let mut buf = [0.0f64; 9];
    for i in 0..n {
        let center = input.points.get(i);
        if tensor_arr.num_components() == 6 {
            let mut sym = [0.0f64; 6];
            tensor_arr.tuple_as_f64(i, &mut sym);
            // VTK's six-component symmetric tensor order is
            // xx, yy, zz, xy, yz, xz.
            buf = [
                sym[0], sym[3], sym[5], sym[3], sym[1], sym[4], sym[5], sym[4], sym[2],
            ];
        } else {
            tensor_arr.tuple_as_f64(i, &mut buf);
        }

        let (eigenvalues, eigenvectors) = symmetric_eigensystem(&buf);
        let scale = nonzero_scales(eigenvalues);
        let col0 = [
            eigenvectors[0][0] * scale[0],
            eigenvectors[1][0] * scale[0],
            eigenvectors[2][0] * scale[0],
        ];
        let col1 = [
            eigenvectors[0][1] * scale[1],
            eigenvectors[1][1] * scale[1],
            eigenvectors[2][1] * scale[1],
        ];
        let col2 = [
            eigenvectors[0][2] * scale[2],
            eigenvectors[1][2] * scale[2],
            eigenvectors[2][2] * scale[2],
        ];

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

fn symmetric_eigensystem(tensor: &[f64; 9]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut a = [
        [
            tensor[0],
            0.5 * (tensor[1] + tensor[3]),
            0.5 * (tensor[2] + tensor[6]),
        ],
        [
            0.5 * (tensor[3] + tensor[1]),
            tensor[4],
            0.5 * (tensor[5] + tensor[7]),
        ],
        [
            0.5 * (tensor[6] + tensor[2]),
            0.5 * (tensor[7] + tensor[5]),
            tensor[8],
        ],
    ];
    let mut v = [[0.0; 3]; 3];
    for (i, row) in v.iter_mut().enumerate() {
        row[i] = 1.0;
    }

    for _ in 0..32 {
        let (p, q, max_off) = largest_off_diagonal(&a);
        if max_off < 1e-12 {
            break;
        }

        let tau = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let sign = if tau >= 0.0 { 1.0 } else { -1.0 };
        let t = sign / (tau.abs() + (1.0 + tau * tau).sqrt());
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        a[p][p] = app - t * apq;
        a[q][q] = aqq + t * apq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let arp = a[r][p];
                let arq = a[r][q];
                a[r][p] = c * arp - s * arq;
                a[p][r] = a[r][p];
                a[r][q] = s * arp + c * arq;
                a[q][r] = a[r][q];
            }

            let vrp = v[r][p];
            let vrq = v[r][q];
            v[r][p] = c * vrp - s * vrq;
            v[r][q] = s * vrp + c * vrq;
        }
    }

    let mut order = [0usize, 1, 2];
    order.sort_by(|&lhs, &rhs| {
        a[rhs][rhs]
            .partial_cmp(&a[lhs][lhs])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut values = [0.0; 3];
    let mut vectors = [[0.0; 3]; 3];
    for (out_col, &src_col) in order.iter().enumerate() {
        values[out_col] = a[src_col][src_col];
        for row in 0..3 {
            vectors[row][out_col] = v[row][src_col];
        }
    }
    (values, vectors)
}

fn largest_off_diagonal(a: &[[f64; 3]; 3]) -> (usize, usize, f64) {
    let pairs = [(0, 1), (0, 2), (1, 2)];
    let mut best = (0, 1, a[0][1].abs());
    for &(p, q) in &pairs[1..] {
        let value = a[p][q].abs();
        if value > best.2 {
            best = (p, q, value);
        }
    }
    best
}

fn nonzero_scales(mut scales: [f64; 3]) -> [f64; 3] {
    let mut max_scale = 0.0f64;
    for &scale in &scales {
        max_scale = max_scale.max(scale.abs());
    }
    if max_scale == 0.0 {
        max_scale = 1.0;
    }
    for scale in &mut scales {
        if *scale == 0.0 {
            *scale = max_scale * 1.0e-6;
        }
    }
    scales
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

    #[test]
    fn six_component_tensor_uses_vtk_order() {
        let mut input = PolyData::new();
        input.points.push([0.0, 0.0, 0.0]);
        let tensor = DataArray::from_vec("T", vec![1.0, 1.0, 1.0, 0.0, 0.0, 2.0], 6);
        input.point_data_mut().add_array(tensor.into());

        let glyph = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );

        let result = tensor_glyph(&input, "T", &glyph);
        assert_eq!(result.points.len(), 3);
    }
}
