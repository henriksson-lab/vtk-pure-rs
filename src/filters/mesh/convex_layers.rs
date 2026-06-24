use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute convex layers (onion peeling) of a 2D point set.
///
/// Iteratively computes convex hulls and removes hull points.
/// Adds "ConvexLayer" scalar (0 = outermost hull, 1 = next, etc.).
pub fn convex_layers_2d(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let pts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            [p[0], p[1]]
        })
        .collect();
    let mut layers = vec![0usize; n];
    let mut remaining: Vec<usize> = (0..n).collect();
    let mut layer = 0;

    while remaining.len() >= 3 {
        let hull_indices = convex_hull_indices(&pts, &remaining);
        if hull_indices.is_empty() {
            break;
        }

        let hull_set: std::collections::HashSet<usize> = hull_indices.iter().copied().collect();
        for &i in &hull_indices {
            layers[i] = layer;
        }

        remaining.retain(|i| !hull_set.contains(i));
        layer += 1;
    }
    // Assign remaining points to last layer
    for &i in &remaining {
        layers[i] = layer;
    }

    let layers_f: Vec<f64> = layers.iter().map(|&l| l as f64).collect();
    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ConvexLayer",
            layers_f,
            1,
        )));
    pd
}

fn convex_hull_indices(pts: &[[f64; 2]], indices: &[usize]) -> Vec<usize> {
    if indices.len() < 3 {
        return indices.to_vec();
    }
    let mut sorted: Vec<usize> = indices.to_vec();
    sorted.sort_by(|&a, &b| {
        pts[a][0]
            .partial_cmp(&pts[b][0])
            .unwrap()
            .then(pts[a][1].partial_cmp(&pts[b][1]).unwrap())
    });

    let cross = |o: usize, a: usize, b: usize| -> f64 {
        (pts[a][0] - pts[o][0]) * (pts[b][1] - pts[o][1])
            - (pts[a][1] - pts[o][1]) * (pts[b][0] - pts[o][0])
    };

    let mut lower = Vec::new();
    for &i in &sorted {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], i) <= 0.0 {
            lower.pop();
        }
        lower.push(i);
    }
    let mut upper = Vec::new();
    for &i in sorted.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], i) <= 0.0 {
            upper.pop();
        }
        upper.push(i);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);

    let mut hull_set = std::collections::HashSet::new();
    for &i in &lower {
        hull_set.insert(i);
    }
    hull_set.into_iter().collect()
}

/// Count the number of convex layers.
pub fn num_convex_layers(input: &PolyData) -> usize {
    let result = convex_layers_2d(input);
    let arr = match result.point_data().get_array("ConvexLayer") {
        Some(a) => a,
        None => return 0,
    };
    let mut buf = [0.0f64];
    let mut max_l = 0usize;
    for i in 0..arr.num_tuples() {
        arr.tuple_as_f64(i, &mut buf);
        max_l = max_l.max(buf[0] as usize);
    }
    max_l + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_with_center() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.0]); // interior point

        let result = convex_layers_2d(&pd);
        let arr = result.point_data().get_array("ConvexLayer").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // hull
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 0.0); // interior layer
    }

    #[test]
    fn num_layers() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.0]);
        pd.points.push([1.5, 0.5, 0.0]);
        pd.points.push([1.5, 1.5, 0.0]);
        pd.points.push([0.5, 1.5, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]); // innermost

        let nl = num_convex_layers(&pd);
        assert!(nl >= 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(num_convex_layers(&pd), 0);
    }
}
