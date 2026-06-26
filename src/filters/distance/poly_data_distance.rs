use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the distance from each point of `source` to the nearest polygon in `target`.
///
/// Adds a "Distance" scalar to `source`'s point data.
pub fn poly_data_distance(source: &PolyData, target: &PolyData) -> PolyData {
    let n_src = source.points.len();
    let triangles = collect_triangles(target);

    if triangles.is_empty() {
        return source.clone();
    }

    let mut distances = vec![0.0f64; n_src];
    for i in 0..n_src {
        let p = source.points.get(i);
        let mut min_d2 = f64::MAX;
        for &(a, b, c) in &triangles {
            min_d2 = min_d2.min(point_triangle_dist2(p, a, b, c));
        }
        distances[i] = min_d2.sqrt();
    }

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    pd.point_data_mut().set_active_scalars("Distance");
    pd
}

/// Compute the symmetric Hausdorff-like distance statistics between two surfaces.
///
/// Returns (max_dist_a_to_b, max_dist_b_to_a, mean_a_to_b, mean_b_to_a).
pub fn distance_stats(a: &PolyData, b: &PolyData) -> (f64, f64, f64, f64) {
    let compute = |src: &PolyData, tgt: &PolyData| -> (f64, f64) {
        let n_src = src.points.len();
        let triangles = collect_triangles(tgt);
        if n_src == 0 || triangles.is_empty() {
            return (0.0, 0.0);
        }

        let mut max_d = 0.0f64;
        let mut sum_d = 0.0f64;
        for i in 0..n_src {
            let p = src.points.get(i);
            let mut min_d2 = f64::MAX;
            for &(a, b, c) in &triangles {
                min_d2 = min_d2.min(point_triangle_dist2(p, a, b, c));
            }
            let d = min_d2.sqrt();
            max_d = max_d.max(d);
            sum_d += d;
        }
        (max_d, sum_d / n_src as f64)
    };

    let (max_ab, mean_ab) = compute(a, b);
    let (max_ba, mean_ba) = compute(b, a);
    (max_ab, max_ba, mean_ab, mean_ba)
}

fn collect_triangles(poly_data: &PolyData) -> Vec<([f64; 3], [f64; 3], [f64; 3])> {
    poly_data
        .polys
        .iter()
        .filter(|cell| cell.len() >= 3)
        .flat_map(|cell| {
            let a = poly_data.points.get(cell[0] as usize);
            (1..cell.len() - 1).map(move |i| {
                (
                    a,
                    poly_data.points.get(cell[i] as usize),
                    poly_data.points.get(cell[i + 1] as usize),
                )
            })
        })
        .collect()
}

fn point_triangle_dist2(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return dist2(p, a);
    }

    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return dist2(p, b);
    }

    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return dist2(p, c);
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return dist2(p, [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]]);
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return dist2(p, [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]]);
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return dist2(
            p,
            [
                b[0] + w * (c[0] - b[0]),
                b[1] + w * (c[1] - b[1]),
                b[2] + w * (c[2] - b[2]),
            ],
        );
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    dist2(
        p,
        [
            a[0] + ab[0] * v + ac[0] * w,
            a[1] + ab[1] * v + ac[1] * w,
            a[2] + ab[2] * v + ac[2] * w,
        ],
    )
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_to_self() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = poly_data_distance(&pd, &pd);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0f64];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn known_distance() {
        let mut src = PolyData::new();
        src.points.push([1.0, 1.0, 5.0]);

        let tgt = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = poly_data_distance(&src, &tgt);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn symmetric_stats() {
        let mut a = PolyData::new();
        a.points.push([1.0, 1.0, 1.0]);
        a.points.push([2.0, 2.0, 1.0]);

        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let (_, _, mean_ab, _mean_ba) = distance_stats(&a, &b);
        assert!((mean_ab - 1.0).abs() < 1e-10);
    }
}
