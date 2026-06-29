use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute tangent vectors for a triangle mesh.
///
/// Mirrors VTK's `vtkPolyDataTangents` triangle path: active texture
/// coordinates are required, polygonal cells must already be triangles, and
/// the output receives an active 3-component point-data "Tangents" array.
pub fn compute_tangents(input: &PolyData) -> PolyData {
    let num_pts = input.points.len();
    if num_pts == 0 {
        return input.clone();
    }
    if input.lines.num_cells() > 0 || input.strips.num_cells() > 0 {
        return input.clone();
    }
    if input.polys.iter().any(|cell| cell.len() != 3) {
        return input.clone();
    }

    let Some(tcoords) = input.point_data().tcoords() else {
        return input.clone();
    };

    let mut tangents = vec![[0.0f64; 3]; num_pts];

    for cell in input.polys.iter() {
        let v1 = input.points.get(cell[0] as usize);
        let v2 = input.points.get(cell[1] as usize);
        let v3 = input.points.get(cell[2] as usize);

        let ax = v3[0] - v2[0];
        let ay = v3[1] - v2[1];
        let az = v3[2] - v2[2];
        let bx = v1[0] - v2[0];
        let by = v1[1] - v2[1];
        let bz = v1[2] - v2[2];

        let mut uv1 = [0.0f64; 2];
        tcoords.tuple_as_f64(cell[0] as usize, &mut uv1);
        let mut uv2 = [0.0f64; 2];
        tcoords.tuple_as_f64(cell[1] as usize, &mut uv2);
        let mut uv3 = [0.0f64; 2];
        tcoords.tuple_as_f64(cell[2] as usize, &mut uv3);

        let duv1_x = uv3[0] - uv2[0];
        let duv1_y = uv3[1] - uv2[1];
        let duv2_x = uv1[0] - uv2[0];
        let duv2_y = uv1[1] - uv2[1];

        let f = 1.0 / (duv1_x * duv2_y - duv2_x * duv1_y);
        let tangent = [
            f * (duv2_y * ax - duv1_y * bx),
            f * (duv2_y * ay - duv1_y * by),
            f * (duv2_y * az - duv1_y * bz),
        ];

        for &id in cell {
            let idx = id as usize;
            tangents[idx][0] += tangent[0];
            tangents[idx][1] += tangent[1];
            tangents[idx][2] += tangent[2];
        }
    }

    for tangent in &mut tangents {
        let len =
            (tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2]).sqrt();
        if len != 0.0 {
            tangent[0] /= len;
            tangent[1] /= len;
            tangent[2] /= len;
        }
    }

    let flat: Vec<f64> = tangents
        .iter()
        .flat_map(|tangent| tangent.iter().copied())
        .collect();
    let mut output = input.clone();
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Tangents", flat, 3)));
    output.point_data_mut().set_active_tangents("Tangents");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tangents() {
        let pd = triangle_with_tcoords();

        let result = compute_tangents(&pd);
        let arr = result.point_data().tangents().unwrap();
        assert_eq!(arr.num_components(), 3);
    }

    #[test]
    fn tangent_direction() {
        let pd = triangle_with_tcoords();

        let result = compute_tangents(&pd);
        let arr = result.point_data().tangents().unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.5);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = compute_tangents(&pd);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn no_tcoords_passes_through() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = compute_tangents(&pd);
        assert!(result.point_data().tangents().is_none());
    }

    fn triangle_with_tcoords() -> PolyData {
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
        pd.point_data_mut().set_active_tcoords("TCoords");
        pd
    }
}
