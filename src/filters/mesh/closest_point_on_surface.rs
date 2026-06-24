use crate::data::{AnyDataArray, DataArray, PolyData};

/// For each point in `source`, find the closest point on the surface of `target`.
///
/// Adds two point data arrays to a clone of `source`:
/// - "ClosestPoint" (3-component): the coordinates of the closest surface point
/// - "Distance" (1-component): the Euclidean distance to that point
///
/// The closest point is computed on actual triangle faces, not just vertices.
pub fn closest_point_on_surface(source: &PolyData, target: &PolyData) -> PolyData {
    let tris = collect_triangles(target);

    let n = source.points.len();
    let mut closest_pts = Vec::with_capacity(n * 3);
    let mut distances = Vec::with_capacity(n);

    for i in 0..n {
        let p = source.points.get(i);
        let mut best_dist2: f64 = f64::MAX;
        let mut best_pt = [0.0f64; 3];

        for (v0, v1, v2) in &tris {
            let cp = closest_point_on_triangle(&p, v0, v1, v2);
            let d2: f64 = (cp[0] - p[0]) * (cp[0] - p[0])
                + (cp[1] - p[1]) * (cp[1] - p[1])
                + (cp[2] - p[2]) * (cp[2] - p[2]);
            if d2 < best_dist2 {
                best_dist2 = d2;
                best_pt = cp;
            }
        }

        closest_pts.extend_from_slice(&best_pt);
        distances.push(best_dist2.sqrt());
    }

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ClosestPoint",
            closest_pts,
            3,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    pd
}

fn collect_triangles(pd: &PolyData) -> Vec<([f64; 3], [f64; 3], [f64; 3])> {
    let mut tris = Vec::new();
    for cell in pd.polys.iter() {
        if cell.len() >= 3 {
            tris.push((
                pd.points.get(cell[0] as usize),
                pd.points.get(cell[1] as usize),
                pd.points.get(cell[2] as usize),
            ));
        }
    }
    tris
}

/// Compute the closest point on triangle (v0, v1, v2) to point p.
/// Uses the parametric projection approach with Voronoi region checks.
fn closest_point_on_triangle(
    p: &[f64; 3],
    v0: &[f64; 3],
    v1: &[f64; 3],
    v2: &[f64; 3],
) -> [f64; 3] {
    let ab = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let ac = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let ap = [p[0] - v0[0], p[1] - v0[1], p[2] - v0[2]];

    let d1: f64 = dot(&ab, &ap);
    let d2: f64 = dot(&ac, &ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return *v0;
    }

    let bp = [p[0] - v1[0], p[1] - v1[1], p[2] - v1[2]];
    let d3: f64 = dot(&ab, &bp);
    let d4: f64 = dot(&ac, &bp);
    if d3 >= 0.0 && d4 <= d3 {
        return *v1;
    }

    let vc: f64 = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v: f64 = d1 / (d1 - d3);
        return [v0[0] + v * ab[0], v0[1] + v * ab[1], v0[2] + v * ab[2]];
    }

    let cp = [p[0] - v2[0], p[1] - v2[1], p[2] - v2[2]];
    let d5: f64 = dot(&ab, &cp);
    let d6: f64 = dot(&ac, &cp);
    if d6 >= 0.0 && d5 <= d6 {
        return *v2;
    }

    let vb: f64 = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w: f64 = d2 / (d2 - d6);
        return [v0[0] + w * ac[0], v0[1] + w * ac[1], v0[2] + w * ac[2]];
    }

    let va: f64 = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w: f64 = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [
            v1[0] + w * (v2[0] - v1[0]),
            v1[1] + w * (v2[1] - v1[1]),
            v1[2] + w * (v2[2] - v1[2]),
        ];
    }

    let denom: f64 = 1.0 / (va + vb + vc);
    let v_param: f64 = vb * denom;
    let w_param: f64 = vc * denom;
    [
        v0[0] + ab[0] * v_param + ac[0] * w_param,
        v0[1] + ab[1] * v_param + ac[1] * w_param,
        v0[2] + ab[2] * v_param + ac[2] * w_param,
    ]
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_on_surface_has_zero_distance() {
        let target = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Source point is exactly on the triangle
        let mut source = PolyData::new();
        source.points.push([1.0, 0.5, 0.0]);

        let result = closest_point_on_surface(&source, &target);
        let dist_arr = result.point_data().get_array("Distance").unwrap();
        let mut d = [0.0f64];
        dist_arr.tuple_as_f64(0, &mut d);
        assert!(d[0] < 1e-10, "distance should be ~0, got {}", d[0]);
    }

    #[test]
    fn point_above_triangle() {
        let target = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut source = PolyData::new();
        source.points.push([1.0, 0.5, 3.0]); // 3 units above the triangle

        let result = closest_point_on_surface(&source, &target);

        let dist_arr = result.point_data().get_array("Distance").unwrap();
        let mut d = [0.0f64];
        dist_arr.tuple_as_f64(0, &mut d);
        assert!(
            (d[0] - 3.0).abs() < 1e-10,
            "distance should be 3.0, got {}",
            d[0]
        );

        let cp_arr = result.point_data().get_array("ClosestPoint").unwrap();
        let mut cp = [0.0f64; 3];
        cp_arr.tuple_as_f64(0, &mut cp);
        assert!(
            (cp[2]).abs() < 1e-10,
            "closest z should be 0, got {}",
            cp[2]
        );
    }

    #[test]
    fn closest_to_edge() {
        let target = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Point is outside the triangle, nearest to the edge from (0,0,0) to (1,0,0)
        let mut source = PolyData::new();
        source.points.push([0.5, -1.0, 0.0]);

        let result = closest_point_on_surface(&source, &target);

        let cp_arr = result.point_data().get_array("ClosestPoint").unwrap();
        let mut cp = [0.0f64; 3];
        cp_arr.tuple_as_f64(0, &mut cp);
        // Closest should be on the edge y=0, at x=0.5
        assert!(
            (cp[0] - 0.5).abs() < 1e-10,
            "x should be 0.5, got {}",
            cp[0]
        );
        assert!(cp[1].abs() < 1e-10, "y should be 0, got {}", cp[1]);

        let dist_arr = result.point_data().get_array("Distance").unwrap();
        let mut d = [0.0f64];
        dist_arr.tuple_as_f64(0, &mut d);
        assert!(
            (d[0] - 1.0).abs() < 1e-10,
            "distance should be 1.0, got {}",
            d[0]
        );
    }
}
