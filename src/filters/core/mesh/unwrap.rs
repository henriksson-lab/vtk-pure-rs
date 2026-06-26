use crate::data::{AnyDataArray, DataArray, DataSet, PolyData};

/// Simple mesh unwrap: project vertices to their UV coordinates.
///
/// If the mesh has "UV" or "TCoords" texture coordinates, creates
/// a new mesh where XY = UV and Z = 0. Useful for visualizing
/// parameterization quality.
pub fn unwrap_to_uv(input: &PolyData) -> PolyData {
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
    let mut points = crate::data::Points::<f64>::new();
    let mut buf = [0.0f64; 2];

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        points.push([buf[0], buf[1], 0.0]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Compute UV distortion: ratio of 3D area to UV area per triangle.
///
/// Adds "UVDistortion" cell data (1.0 = no distortion, >1 or <1 = stretched/compressed).
pub fn uv_distortion(input: &PolyData) -> PolyData {
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

    let mut distortion = Vec::new();
    let mut buf = [0.0f64; 2];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            distortion.push(1.0);
            continue;
        }
        if cell
            .iter()
            .any(|&pid| pid as usize >= input.points.len() || pid as usize >= arr.num_tuples())
        {
            distortion.push(1.0);
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);

        // 3D area
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let c = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let area_3d = 0.5 * (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();

        // UV area
        arr.tuple_as_f64(cell[0] as usize, &mut buf);
        let u0 = [buf[0], buf[1]];
        arr.tuple_as_f64(cell[1] as usize, &mut buf);
        let u1 = [buf[0], buf[1]];
        arr.tuple_as_f64(cell[2] as usize, &mut buf);
        let u2 = [buf[0], buf[1]];
        let area_uv =
            0.5 * ((u1[0] - u0[0]) * (u2[1] - u0[1]) - (u2[0] - u0[0]) * (u1[1] - u0[1])).abs();

        let ratio = if area_uv > 1e-15 {
            area_3d / area_uv
        } else {
            0.0
        };
        distortion.push(ratio);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "UVDistortion",
            distortion,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwrap_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 1.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.points.push([0.0, 1.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                2,
            )));

        let result = unwrap_to_uv(&pd);
        let p = result.points.get(0);
        assert_eq!(p[2], 0.0); // flattened
        assert_eq!(p[0], 0.0); // UV x
    }

    #[test]
    fn distortion_uniform() {
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

        let result = uv_distortion(&pd);
        let arr = result.cell_data().get_array("UVDistortion").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.1); // ~1:1 mapping
    }

    #[test]
    fn no_tcoords() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = unwrap_to_uv(&pd);
        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]); // unchanged
    }
}
