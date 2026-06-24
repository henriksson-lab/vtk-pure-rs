use crate::data::{Points, PolyData};

/// Scalar-weighted Laplacian smoothing.
///
/// Each vertex moves toward its neighbor average, but the step size
/// is modulated by the named scalar array. High scalar = more smoothing,
/// low scalar = less (preserves features where scalar is low).
pub fn weighted_smooth(
    input: &PolyData,
    weight_array: &str,
    lambda: f64,
    iterations: usize,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let weights: Vec<f64> = if let Some(arr) = input.point_data().get_array(weight_array) {
        let mut buf = [0.0f64];
        (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0].clamp(0.0, 1.0)
            })
            .collect()
    } else {
        vec![1.0; n]
    };

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pts = pts.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let w = weights[i] * lambda;
            let cnt = neighbors[i].len() as f64;
            let mut ax = 0.0;
            let mut ay = 0.0;
            let mut az = 0.0;
            for &j in &neighbors[i] {
                ax += pts[j][0];
                ay += pts[j][1];
                az += pts[j][2];
            }
            new_pts[i] = [
                pts[i][0] + w * (ax / cnt - pts[i][0]),
                pts[i][1] + w * (ay / cnt - pts[i][1]),
                pts[i][2] + w * (az / cnt - pts[i][2]),
            ];
        }
        pts = new_pts;
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn high_weight_smooths() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 2.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![1.0, 1.0, 1.0],
                1,
            )));

        let result = weighted_smooth(&pd, "w", 0.5, 5);
        let p = result.points.get(2);
        assert!(p[2] < 2.0); // smoothed
    }

    #[test]
    fn zero_weight_preserves() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 5.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "w",
                vec![0.0, 0.0, 0.0],
                1,
            )));

        let result = weighted_smooth(&pd, "w", 0.5, 10);
        let p = result.points.get(2);
        assert!((p[2] - 5.0).abs() < 1e-10); // preserved
    }

    #[test]
    fn missing_array_uses_uniform() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = weighted_smooth(&pd, "nope", 0.5, 5);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = weighted_smooth(&pd, "w", 0.5, 5);
        assert_eq!(result.points.len(), 0);
    }
}
