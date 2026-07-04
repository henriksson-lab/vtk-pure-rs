//! Classify mesh vertices and faces relative to another mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn classify_vertices(mesh: &PolyData, reference: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            if point_inside([p[0] + 1e-7, p[1] + 1.3e-7, p[2] + 0.9e-7], reference) {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("InsideRef", data, 1)));
    r
}
pub fn vertex_distance_to_mesh(mesh: &PolyData, reference: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            min_distance_to_surface(p, reference)
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("DistToRef", data, 1)));
    r.point_data_mut().set_active_scalars("DistToRef");
    r
}

fn min_distance_to_surface(p: [f64; 3], mesh: &PolyData) -> f64 {
    let mut best = f64::INFINITY;
    let n = mesh.points.len();

    for cell in mesh.polys.iter() {
        if cell.len() < 3
            || !cell
                .iter()
                .all(|&id| usize::try_from(id).is_ok_and(|idx| idx < n))
        {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            best = best.min(point_triangle_dist2(p, a, b, c));
        }
    }

    if best.is_finite() {
        return best.sqrt();
    }

    for i in 0..n {
        let q = mesh.points.get(i);
        best = best.min(dist2(p, q));
    }
    best.sqrt()
}

fn point_inside(p: [f64; 3], mesh: &PolyData) -> bool {
    let mut crossings = 0;
    let n = mesh.points.len();
    for cell in mesh.polys.iter() {
        if cell.len() < 3
            || !cell
                .iter()
                .all(|&id| usize::try_from(id).is_ok_and(|idx| idx < n))
        {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let h = [
                0.0 * e2[2] - 0.0 * e2[1],
                0.0 * e2[0] - 1.0 * e2[2],
                1.0 * e2[1] - 0.0 * e2[0],
            ]; // dir=[1,0,0]
            let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
            if det.abs() < 1e-12 {
                continue;
            }
            let inv = 1.0 / det;
            let s = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
            let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
            if u < 0.0 || u > 1.0 {
                continue;
            }
            let q = [
                s[1] * e1[2] - s[2] * e1[1],
                s[2] * e1[0] - s[0] * e1[2],
                s[0] * e1[1] - s[1] * e1[0],
            ];
            let v = inv * (1.0 * q[0] + 0.0 * q[1] + 0.0 * q[2]);
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
            if t > 1e-12 {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
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

    let denom = va + vb + vc;
    if denom.abs() <= 1e-15 {
        return dist2(p, a).min(dist2(p, b)).min(dist2(p, c));
    }
    let inv = 1.0 / denom;
    let v = vb * inv;
    let w = vc * inv;
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
    fn test_classify() {
        let inner = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.05, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let outer = PolyData::from_triangles(
            vec![
                [-5.0, -5.0, -5.0],
                [5.0, -5.0, -5.0],
                [0.0, 5.0, -5.0],
                [0.0, 0.0, 5.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = classify_vertices(&inner, &outer);
        assert!(r.point_data().get_array("InsideRef").is_some());
    }
    #[test]
    fn test_distance() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = vertex_distance_to_mesh(&a, &b);
        let mut buf = [0.0];
        r.point_data()
            .get_array("DistToRef")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
}
