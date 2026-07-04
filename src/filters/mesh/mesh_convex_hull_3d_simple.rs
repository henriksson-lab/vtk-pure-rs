//! Simple 3D convex hull using gift wrapping on projected axes.
use crate::data::{CellArray, Points, PolyData};

pub fn convex_hull_3d_approx(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n < 4 {
        return mesh.clone();
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let mut hull_faces: Vec<[usize; 3]> = Vec::new();
    let mut seen_faces = std::collections::HashSet::new();
    const EPS: f64 = 1e-10;

    for i in 0..n {
        for j in i + 1..n {
            for k in j + 1..n {
                let u = sub(pts[j], pts[i]);
                let v = sub(pts[k], pts[i]);
                let normal = cross(u, v);
                if dot(normal, normal) <= EPS * EPS {
                    continue;
                }

                let mut pos = false;
                let mut neg = false;
                for m in 0..n {
                    if m == i || m == j || m == k {
                        continue;
                    }
                    let d = dot(normal, sub(pts[m], pts[i]));
                    if d > EPS {
                        pos = true;
                    } else if d < -EPS {
                        neg = true;
                    }
                    if pos && neg {
                        break;
                    }
                }
                if pos && neg {
                    continue;
                }

                let mut key = [i, j, k];
                key.sort_unstable();
                if seen_faces.insert(key) {
                    if pos {
                        hull_faces.push([i, k, j]);
                    } else {
                        hull_faces.push([i, j, k]);
                    }
                }
            }
        }
    }

    if hull_faces.is_empty() {
        return mesh.clone();
    }

    let mut point_map = std::collections::HashMap::new();
    let mut new_pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for face in hull_faces {
        let mut out = [0i64; 3];
        for (dst, src) in out.iter_mut().zip(face) {
            let idx = *point_map.entry(src).or_insert_with(|| {
                let idx = new_pts.len();
                new_pts.push(pts[src]);
                idx
            });
            *dst = idx as i64;
        }
        polys.push_cell(&out);
    }
    let mut r = PolyData::new();
    r.points = new_pts;
    r.polys = polys;
    r
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.points.push([0.0, 0.0, 1.0]);
        m.points.push([0.3, 0.3, 0.3]);
        let r = convex_hull_3d_approx(&m);
        assert_eq!(r.polys.num_cells(), 4);
        assert_eq!(r.points.len(), 4);
    }
}
