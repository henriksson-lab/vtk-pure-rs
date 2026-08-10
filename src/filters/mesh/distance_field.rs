use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute geodesic-like distance from a set of seed points along mesh edges.
///
/// Multi-seed form of
/// [`crate::filters::mesh::geodesic_distance_dijkstra::geodesic_distance`],
/// which holds the single Dijkstra implementation: the result is the pointwise
/// minimum over the single-source fields of each seed. `seed_indices` are the
/// starting points (distance = 0); unreachable vertices get -1.0. Adds a
/// "GeodesicDistance" scalar.
pub fn geodesic_distance(input: &PolyData, seed_indices: &[usize]) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut dist = vec![-1.0f64; n];
    let mut buf = [0.0f64];
    for &seed in seed_indices {
        if seed >= n {
            continue;
        }
        let field =
            crate::filters::mesh::geodesic_distance_dijkstra::geodesic_distance(input, seed);
        let Some(arr) = field.point_data().get_array("GeodesicDistance") else {
            continue;
        };
        for (i, slot) in dist.iter_mut().enumerate() {
            arr.tuple_as_f64(i, &mut buf);
            let d = buf[0];
            if d >= 0.0 && (*slot < 0.0 || d < *slot) {
                *slot = d;
            }
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "GeodesicDistance",
            dist,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_from_corner() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = geodesic_distance(&pd, &[0]);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn multiple_seeds() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([2.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = geodesic_distance(&pd, &[0, 1]);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn empty_seeds() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = geodesic_distance(&pd, &[]);
        let arr = result.point_data().get_array("GeodesicDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], -1.0); // unreachable
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = geodesic_distance(&pd, &[0]);
        assert_eq!(result.points.len(), 0);
    }
}
