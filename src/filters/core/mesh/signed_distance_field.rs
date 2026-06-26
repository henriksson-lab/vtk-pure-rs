use crate::data::{AnyDataArray, DataArray, DataSet, ImageData, PolyData};

/// Compute a signed distance field on an ImageData grid from a closed PolyData surface.
///
/// For each voxel center, the unsigned distance to the nearest triangle is computed.
/// The sign is determined by closed-surface parity: negative inside and positive outside.
pub fn compute_sdf(input: &PolyData, dims: [u32; 3], bounds: [[f64; 2]; 3]) -> ImageData {
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;

    let spacing: [f64; 3] = [
        if nx > 1 {
            (bounds[0][1] - bounds[0][0]) / (nx - 1) as f64
        } else {
            1.0
        },
        if ny > 1 {
            (bounds[1][1] - bounds[1][0]) / (ny - 1) as f64
        } else {
            1.0
        },
        if nz > 1 {
            (bounds[2][1] - bounds[2][0]) / (nz - 1) as f64
        } else {
            1.0
        },
    ];
    let origin: [f64; 3] = [bounds[0][0], bounds[1][0], bounds[2][0]];

    let mut image = ImageData::with_dimensions(nx, ny, nz);
    image.set_spacing(spacing);
    image.set_origin(origin);

    // Collect triangles with their face normals
    let mut tris: Vec<([f64; 3], [f64; 3], [f64; 3], [f64; 3])> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() >= 3 {
            let p0 = input.points.get(cell[0] as usize);
            for i in 1..cell.len() - 1 {
                let p1 = input.points.get(cell[i] as usize);
                let p2 = input.points.get(cell[i + 1] as usize);
                let e1 = sub(p1, p0);
                let e2 = sub(p2, p0);
                let n = cross(e1, e2);
                let len: f64 = length(n);
                let normal = if len > 1e-20 {
                    [n[0] / len, n[1] / len, n[2] / len]
                } else {
                    [0.0, 0.0, 1.0]
                };
                tris.push((p0, p1, p2, normal));
            }
        }
    }

    let n_points: usize = image.num_points();
    let mut sdf_values: Vec<f64> = vec![0.0; n_points];

    for idx in 0..n_points {
        let p = image.point(idx);
        let mut min_dist2: f64 = f64::MAX;
        for &(v0, v1, v2, _) in &tris {
            let (d2, _) = point_triangle_closest(p, v0, v1, v2);
            if d2 < min_dist2 {
                min_dist2 = d2;
            }
        }

        let dist: f64 = min_dist2.sqrt();
        let sign: f64 = if point_inside_closed_surface(p, &tris) {
            -1.0
        } else {
            1.0
        };
        sdf_values[idx] = sign * dist;
    }

    let arr = DataArray::from_vec("SDF", sdf_values, 1);
    image.point_data_mut().add_array(AnyDataArray::F64(arr));
    image.point_data_mut().set_active_scalars("SDF");
    image
}

/// Returns (squared_distance, closest_point_on_triangle).
fn point_triangle_closest(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> (f64, [f64; 3]) {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);

    let d1: f64 = dot(ab, ap);
    let d2: f64 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (dist2(p, a), a);
    }

    let bp = sub(p, b);
    let d3: f64 = dot(ab, bp);
    let d4: f64 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (dist2(p, b), b);
    }

    let cp = sub(p, c);
    let d5: f64 = dot(ab, cp);
    let d6: f64 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (dist2(p, c), c);
    }

    let vc: f64 = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v: f64 = d1 / (d1 - d3);
        let proj = [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
        return (dist2(p, proj), proj);
    }

    let vb: f64 = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w: f64 = d2 / (d2 - d6);
        let proj = [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
        return (dist2(p, proj), proj);
    }

    let va: f64 = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w: f64 = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let proj = [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
        return (dist2(p, proj), proj);
    }

    let denom: f64 = 1.0 / (va + vb + vc);
    let v: f64 = vb * denom;
    let w: f64 = vc * denom;
    let proj = [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ];
    (dist2(p, proj), proj)
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

fn point_inside_closed_surface(
    p: [f64; 3],
    tris: &[([f64; 3], [f64; 3], [f64; 3], [f64; 3])],
) -> bool {
    let dir = [1.0, 0.0, 0.0];
    let mut hits: Vec<f64> = Vec::new();

    for &(a, b, c, _) in tris {
        if let Some(t) = ray_triangle_intersection(p, dir, a, b, c) {
            if t > 1e-12 && !hits.iter().any(|&u| (u - t).abs() < 1e-9) {
                hits.push(t);
            }
        }
    }

    hits.len() % 2 == 1
}

fn ray_triangle_intersection(
    origin: [f64; 3],
    dir: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> Option<f64> {
    let eps = 1e-12;
    let edge1 = sub(b, a);
    let edge2 = sub(c, a);
    let h = cross(dir, edge2);
    let det = dot(edge1, h);
    if det.abs() < eps {
        return None;
    }

    let inv_det = 1.0 / det;
    let s = sub(origin, a);
    let u = inv_det * dot(s, h);
    if u < -eps || u > 1.0 + eps {
        return None;
    }

    let q = cross(s, edge1);
    let v = inv_det * dot(dir, q);
    if v < -eps || u + v > 1.0 + eps {
        return None;
    }

    let t = inv_det * dot(edge2, q);
    (t > eps).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_unit_cube() -> PolyData {
        // A simple closed cube from -0.5 to 0.5
        let pts: Vec<[f64; 3]> = vec![
            [-0.5, -0.5, -0.5], // 0
            [0.5, -0.5, -0.5],  // 1
            [0.5, 0.5, -0.5],   // 2
            [-0.5, 0.5, -0.5],  // 3
            [-0.5, -0.5, 0.5],  // 4
            [0.5, -0.5, 0.5],   // 5
            [0.5, 0.5, 0.5],    // 6
            [-0.5, 0.5, 0.5],   // 7
        ];
        // 12 triangles (2 per face), outward-facing normals
        let tris: Vec<[i64; 3]> = vec![
            // -Z face
            [0, 2, 1],
            [0, 3, 2],
            // +Z face
            [4, 5, 6],
            [4, 6, 7],
            // -Y face
            [0, 1, 5],
            [0, 5, 4],
            // +Y face
            [2, 3, 7],
            [2, 7, 6],
            // -X face
            [0, 4, 7],
            [0, 7, 3],
            // +X face
            [1, 2, 6],
            [1, 6, 5],
        ];
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn sdf_dimensions_match() {
        let cube = make_unit_cube();
        let dims: [u32; 3] = [5, 5, 5];
        let bounds: [[f64; 2]; 3] = [[-1.0, 1.0], [-1.0, 1.0], [-1.0, 1.0]];
        let image = compute_sdf(&cube, dims, bounds);
        assert_eq!(image.dimensions(), [5, 5, 5]);
        let arr = image.point_data().get_array("SDF").unwrap();
        assert_eq!(arr.num_tuples(), 125);
    }

    #[test]
    fn sdf_center_is_negative() {
        let cube = make_unit_cube();
        let dims: [u32; 3] = [3, 3, 3];
        let bounds: [[f64; 2]; 3] = [[-1.0, 1.0], [-1.0, 1.0], [-1.0, 1.0]];
        let image = compute_sdf(&cube, dims, bounds);
        // The center voxel (1,1,1) is at (0,0,0), inside the cube -> negative
        let arr = image.point_data().get_array("SDF").unwrap();
        let center_idx: usize = 1 * 3 * 3 + 1 * 3 + 1; // z*ny*nx + y*nx + x
        let mut buf = [0.0f64];
        // Try the index that corresponds to (0,0,0)
        // With dims [3,3,3] and bounds [-1,1], points are at -1, 0, 1.
        // Index ordering: x varies fastest. center = x=1,y=1,z=1 = 1+1*3+1*9=13
        let idx: usize = 1 + 1 * 3 + 1 * 9;
        arr.tuple_as_f64(idx, &mut buf);
        assert!(
            buf[0] < 0.0,
            "Center should be negative (inside), got {}",
            buf[0]
        );
        let _ = center_idx; // suppress unused warning
    }

    #[test]
    fn sdf_corner_is_positive() {
        let cube = make_unit_cube();
        let dims: [u32; 3] = [3, 3, 3];
        let bounds: [[f64; 2]; 3] = [[-1.0, 1.0], [-1.0, 1.0], [-1.0, 1.0]];
        let image = compute_sdf(&cube, dims, bounds);
        // Corner voxel (0,0,0) is at (-1,-1,-1), well outside the cube -> positive
        let arr = image.point_data().get_array("SDF").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] > 0.0,
            "Corner should be positive (outside), got {}",
            buf[0]
        );
    }
}
