use crate::data::{AnyDataArray, CellArray, DataArray, KdTree, Points, PolyData};

/// Sample a scalar field onto a spherical probe.
///
/// Creates a sphere of given radius/resolution centered at `center`,
/// then for each sphere point finds the nearest input point and copies
/// its scalar value. Useful for radial sampling patterns.
pub fn sample_on_sphere(
    input: &PolyData,
    array_name: &str,
    center: [f64; 3],
    radius: f64,
    resolution: usize,
) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let res = resolution.max(4);
    let n_phi = res;
    let n_theta = res * 2;

    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let mut values = Vec::new();
    let mut buf = [0.0f64];

    for j in 0..=n_phi {
        let phi = std::f64::consts::PI * j as f64 / n_phi as f64;
        let sp = phi.sin();
        let cp = phi.cos();

        let n_at_phi = if j == 0 || j == n_phi { 1 } else { n_theta };

        for i in 0..n_at_phi {
            let theta = 2.0 * std::f64::consts::PI * i as f64 / n_at_phi as f64;
            let p = [
                center[0] + radius * sp * theta.cos(),
                center[1] + radius * sp * theta.sin(),
                center[2] + radius * cp,
            ];

            let idx = out_points.len() as i64;
            out_points.push(p);
            out_verts.push_cell(&[idx]);

            if let Some((nearest, _)) = tree.nearest(p) {
                arr.tuple_as_f64(nearest, &mut buf);
                values.push(buf[0]);
            } else {
                values.push(0.0);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_basic() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![10.0, 20.0, 30.0],
                1,
            )));

        let result = sample_on_sphere(&pd, "val", [0.0, 0.0, 0.0], 2.0, 4);
        assert!(result.points.len() > 5);
        assert!(result.point_data().get_array("val").is_some());
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = sample_on_sphere(&pd, "nope", [0.0; 3], 1.0, 4);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn poles_exist() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0], 1)));

        let result = sample_on_sphere(&pd, "v", [0.0, 0.0, 0.0], 1.0, 4);
        // Check north and south pole exist
        let north = result.points.get(0);
        assert!((north[2] - 1.0).abs() < 1e-10);
    }
}
