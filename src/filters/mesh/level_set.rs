use crate::data::{CellArray, Points, PolyData};

/// Extract a level set (isocontour) of a scalar field on a triangle mesh.
///
/// Finds edges where the scalar crosses the isovalue and interpolates
/// the crossing point. Returns a PolyData with line segments.
pub fn mesh_level_set(input: &PolyData, array_name: &str, isovalue: f64) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let mut buf = [0.0f64];
    let n = input.points.len();
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut out_pts = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        let sv = [values[ids[0]], values[ids[1]], values[ids[2]]];

        let mut crossing_pts: Vec<[f64; 3]> = Vec::new();
        for k in 0..3 {
            let a = ids[k];
            let b = ids[(k + 1) % 3];
            let sa = sv[k];
            let sb = sv[(k + 1) % 3];
            if (sa - isovalue) * (sb - isovalue) < 0.0 {
                let t = (isovalue - sa) / (sb - sa);
                let pa = input.points.get(a);
                let pb = input.points.get(b);
                crossing_pts.push([
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    pa[2] + t * (pb[2] - pa[2]),
                ]);
            }
        }

        if crossing_pts.len() == 2 {
            let i0 = out_pts.len() as i64;
            out_pts.push(crossing_pts[0]);
            out_pts.push(crossing_pts[1]);
            out_lines.push_cell(&[i0, i0 + 1]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn contour_on_gradient() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 2.0, 1.0, 0.0],
                1,
            )));

        let result = mesh_level_set(&pd, "f", 0.5);
        assert!(result.lines.num_cells() > 0);
    }

    #[test]
    fn no_crossing() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![5.0, 5.0, 5.0],
                1,
            )));

        let result = mesh_level_set(&pd, "f", 0.0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = mesh_level_set(&pd, "nope", 0.0);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
