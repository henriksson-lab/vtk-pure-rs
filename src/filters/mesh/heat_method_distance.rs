use crate::data::{AnyDataArray, DataArray, PolyData};

/// Approximate geodesic distance using the heat method, from a set of source vertices.
///
/// Thin multi-source wrapper around
/// [`crate::filters::mesh::mesh_heat_method_geodesic::heat_method_distance`], which owns
/// the actual heat-method solve (heat diffusion, normalised gradient, Poisson recovery).
/// Each source is solved independently and the per-vertex minimum is kept, so the result
/// is the distance to the nearest source.
///
/// Adds a "HeatDistance" scalar. Vertices are reported as `-1.0` when no usable source
/// vertex was supplied.
pub fn heat_method_distance(input: &PolyData, sources: &[usize], diffusion_time: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let dt = diffusion_time.max(0.01);
    let heat_steps = (dt / 0.1).ceil() as usize;

    let mut nearest: Option<Vec<f64>> = None;
    for &source in sources.iter().filter(|&&s| s < n) {
        let solved = crate::filters::mesh::mesh_heat_method_geodesic::heat_method_distance(
            input, source, dt, heat_steps,
        );
        let d = solved
            .point_data()
            .get_array("HeatGeodesic")
            .map(|arr| arr.to_f64_vec())
            .unwrap_or_else(|| vec![0.0; n]);
        nearest = Some(match nearest {
            None => d,
            Some(best) => best.iter().zip(d.iter()).map(|(&a, &b)| a.min(b)).collect(),
        });
    }

    let result = nearest.unwrap_or_else(|| vec![-1.0; n]);

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HeatDistance",
            result,
            1,
        )));
    pd.point_data_mut().set_active_scalars("HeatDistance");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_zero_distance() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = heat_method_distance(&pd, &[0], 1.0);
        let arr = result.point_data().get_array("HeatDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0]).abs() < 1e-5); // source = 0
    }

    #[test]
    fn multiple_sources_take_the_nearest() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let single = heat_method_distance(&mesh, &[0], 1.0);
        let multi = heat_method_distance(&mesh, &[0, 3], 1.0);
        let a = single
            .point_data()
            .get_array("HeatDistance")
            .unwrap()
            .to_f64_vec();
        let b = multi
            .point_data()
            .get_array("HeatDistance")
            .unwrap()
            .to_f64_vec();
        for i in 0..4 {
            assert!(
                b[i] <= a[i] + 1e-12,
                "adding a source must not increase distance at vertex {i}"
            );
        }
    }

    #[test]
    fn no_usable_source_marks_every_vertex() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = heat_method_distance(&mesh, &[99], 1.0);
        let d = result
            .point_data()
            .get_array("HeatDistance")
            .unwrap()
            .to_f64_vec();
        assert_eq!(d, vec![-1.0; 3]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = heat_method_distance(&pd, &[0], 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
