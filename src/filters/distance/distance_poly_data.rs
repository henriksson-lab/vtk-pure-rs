use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the minimum distance from each point in `target` to the
/// surface of `source`.
///
/// Uses brute-force point-to-triangle distance. Adds a "Distance" scalar
/// array to the target's point data.
pub fn distance_poly_data(source: &PolyData, target: &PolyData) -> PolyData {
    let mut pd = target.clone();
    let n = target.points.len();

    // Collect source triangles
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3])> = source
        .polys
        .iter()
        .flat_map(|cell| {
            let p0 = source.points.get(cell[0] as usize);
            (1..cell.len() - 1).map(move |i| {
                (
                    p0,
                    source.points.get(cell[i] as usize),
                    source.points.get(cell[i + 1] as usize),
                )
            })
        })
        .collect();

    let mut distances = Vec::with_capacity(n);
    for i in 0..n {
        let p = target.points.get(i);
        let mut min_d2 = f64::MAX;
        for &(a, b, c) in &tris {
            let d2 = point_triangle_dist2(p, a, b, c);
            if d2 < min_d2 {
                min_d2 = d2;
            }
        }
        distances.push(min_d2.sqrt());
    }

    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    pd
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
    fn distance_from_plane() {
        let source = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut target = PolyData::new();
        target.points.push([1.0, 1.0, 5.0]); // 5 units above triangle

        let result = distance_poly_data(&source, &target);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn distance_zero_on_surface() {
        let source = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut target = PolyData::new();
        target.points.push([0.5, 0.3, 0.0]); // on the triangle

        let result = distance_poly_data(&source, &target);
        let arr = result.point_data().get_array("Distance").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!(val[0] < 1e-10);
    }
}
