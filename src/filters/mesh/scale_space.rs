use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute difference-of-Gaussians (DoG) on a mesh scalar field.
///
/// Smooths the scalar at two scales and takes the difference.
/// High values indicate features at the scale between sigma1 and sigma2.
/// Adds "DoG" scalar array.
pub fn difference_of_gaussians_mesh(
    input: &PolyData,
    array_name: &str,
    sigma1_iters: usize,
    sigma2_iters: usize,
) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    if n == 0 {
        return input.clone();
    }

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

    let mut buf = [0.0f64];
    let orig: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let smooth = |iters: usize| -> Vec<f64> {
        let mut v = orig.clone();
        for _ in 0..iters {
            let mut new = v.clone();
            for i in 0..n {
                if neighbors[i].is_empty() {
                    continue;
                }
                let avg: f64 =
                    neighbors[i].iter().map(|&j| v[j]).sum::<f64>() / neighbors[i].len() as f64;
                new[i] = v[i] + 0.3 * (avg - v[i]);
            }
            v = new;
        }
        v
    };

    let s1 = smooth(sigma1_iters);
    let s2 = smooth(sigma2_iters);
    let dog: Vec<f64> = (0..n).map(|i| s1[i] - s2[i]).collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("DoG", dog, 1)));
    pd
}

/// Detect scale-space extrema (features) from DoG field.
/// Returns indices of vertices that are local extrema in DoG.
pub fn detect_scale_space_features(
    input: &PolyData,
    array_name: &str,
    sigma1: usize,
    sigma2: usize,
    threshold: f64,
) -> Vec<usize> {
    let result = difference_of_gaussians_mesh(input, array_name, sigma1, sigma2);
    let arr = match result.point_data().get_array("DoG") {
        Some(a) => a,
        None => return vec![],
    };

    let n = input.points.len();
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

    let mut buf = [0.0f64];
    let dog: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut features = Vec::new();
    for i in 0..n {
        if dog[i].abs() < threshold {
            continue;
        }
        let is_max = neighbors[i].iter().all(|&j| dog[i] > dog[j]);
        let is_min = neighbors[i].iter().all(|&j| dog[i] < dog[j]);
        if is_max || is_min {
            features.push(i);
        }
    }
    features
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dog_basic() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                (0..9).map(|i| i as f64).collect(),
                1,
            )));

        let result = difference_of_gaussians_mesh(&pd, "v", 1, 5);
        assert!(result.point_data().get_array("DoG").is_some());
    }

    #[test]
    fn uniform_zero_dog() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 3], 1)));

        let result = difference_of_gaussians_mesh(&pd, "v", 2, 6);
        let arr = result.point_data().get_array("DoG").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0].abs() < 1e-10);
        }
    }

    #[test]
    fn feature_detection() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center: high value
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                1,
            )));

        let features = detect_scale_space_features(&pd, "v", 1, 3, 0.01);
        assert!(!features.is_empty()); // center should be detected
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = difference_of_gaussians_mesh(&pd, "nope", 1, 3);
        assert_eq!(result.points.len(), 0);
    }
}
