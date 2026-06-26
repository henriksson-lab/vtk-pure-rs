use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate a checkerboard pattern from UV coordinates.
///
/// Creates a "Checker" scalar (0 or 1) based on UV texture coordinates
/// with configurable frequency. Useful for visualizing UV quality.
pub fn uv_checkerboard(input: &PolyData, frequency: f64) -> PolyData {
    let tc = input
        .point_data()
        .tcoords()
        .or_else(|| input.point_data().get_array("TCoords"))
        .or_else(|| input.point_data().get_array("UV"))
        .or_else(|| input.point_data().get_array("AtlasUV"));
    let arr = match tc {
        Some(a) if a.num_components() == 2 => a,
        _ => return input.clone(),
    };

    let n = input.points.len();
    if arr.num_tuples() < n {
        return input.clone();
    }
    let mut buf = [0.0f64; 2];
    let checker: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            let u = (buf[0] * frequency).floor() as i64;
            let v = (buf[1] * frequency).floor() as i64;
            if (u + v) % 2 == 0 {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Checker", checker, 1,
        )));
    pd
}

/// Compute UV seam length: total length of edges where UV is discontinuous.
pub fn uv_seam_length(input: &PolyData) -> f64 {
    let tc = input
        .point_data()
        .tcoords()
        .or_else(|| input.point_data().get_array("TCoords"))
        .or_else(|| input.point_data().get_array("UV"))
        .or_else(|| input.point_data().get_array("AtlasUV"));
    let arr = match tc {
        Some(a) if a.num_components() == 2 => a,
        _ => return 0.0,
    };

    let mut total = 0.0;
    let mut seen = std::collections::HashSet::new();
    let mut buf_a = [0.0f64; 2];
    let mut buf_b = [0.0f64; 2];

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            if a as usize >= input.points.len()
                || b as usize >= input.points.len()
                || a as usize >= arr.num_tuples()
                || b as usize >= arr.num_tuples()
            {
                continue;
            }
            let key = if a < b { (a, b) } else { (b, a) };
            if !seen.insert(key) {
                continue;
            }

            arr.tuple_as_f64(a as usize, &mut buf_a);
            arr.tuple_as_f64(b as usize, &mut buf_b);
            let uv_dist = ((buf_a[0] - buf_b[0]).powi(2) + (buf_a[1] - buf_b[1]).powi(2)).sqrt();
            let pa = input.points.get(a as usize);
            let pb = input.points.get(b as usize);
            let xyz_dist =
                ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                    .sqrt();

            // UV seam = large UV distance relative to spatial distance
            if xyz_dist > 1e-15 && uv_dist / xyz_dist > 2.0 {
                total += xyz_dist;
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checker_pattern() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                2,
            )));

        let result = uv_checkerboard(&pd, 2.0);
        assert!(result.point_data().get_array("Checker").is_some());
    }

    #[test]
    fn no_tcoords() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = uv_checkerboard(&pd, 1.0);
        assert!(result.point_data().get_array("Checker").is_none());
    }

    #[test]
    fn seam_length_no_seams() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![0.0, 0.0, 0.1, 0.0, 0.05, 0.1],
                2,
            )));

        let len = uv_seam_length(&pd);
        assert_eq!(len, 0.0); // no seams if UV is proportional
    }
}
