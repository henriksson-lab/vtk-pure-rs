use crate::data::PolyData;

/// Compute the largest inscribed sphere in a convex mesh.
///
/// Uses an iterative approach: start at the centroid, then compute the distance
/// to every face plane. The inscribed radius is the minimum distance to any face.
/// The center is iteratively adjusted toward the face that is farthest from the
/// current minimum to maximize the inscribed radius.
///
/// Returns `(radius, center)`.
pub fn inscribed_sphere(input: &PolyData) -> (f64, [f64; 3]) {
    let n = input.points.len();
    if n == 0 {
        return (0.0, [0.0, 0.0, 0.0]);
    }

    // Compute centroid
    let mut cx: f64 = 0.0;
    let mut cy: f64 = 0.0;
    let mut cz: f64 = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;

    // Collect face planes (normal + offset)
    let planes = collect_face_planes(input);
    if planes.is_empty() {
        return (0.0, [cx, cy, cz]);
    }

    let mut center = [cx, cy, cz];
    let iterations: usize = 200;
    let mut step_size: f64 = 0.1;

    for _ in 0..iterations {
        let (min_dist, min_idx) = min_signed_distance(&planes, &center);
        if min_dist < 0.0 {
            // Center is outside; push it inward along the violating plane's normal
            let (nx, ny, nz, _) = planes[min_idx];
            center[0] += nx * (-min_dist + 1e-12);
            center[1] += ny * (-min_dist + 1e-12);
            center[2] += nz * (-min_dist + 1e-12);
            continue;
        }

        // Try to improve: move toward each face direction and check if min distance improves
        let mut best_center = center;
        let mut best_dist = min_dist;

        for plane in &planes {
            let (nx, ny, nz, _) = *plane;
            // Try moving opposite to the normal (toward the face)
            let trial = [
                center[0] - nx * step_size,
                center[1] - ny * step_size,
                center[2] - nz * step_size,
            ];
            let (d, _) = min_signed_distance(&planes, &trial);
            if d > best_dist {
                best_dist = d;
                best_center = trial;
            }
        }

        if best_dist > min_dist + 1e-14 {
            center = best_center;
        } else {
            step_size *= 0.5;
        }
    }

    let (radius, _) = min_signed_distance(&planes, &center);
    (radius.max(0.0), center)
}

/// Compute the signed distance from a point to each face plane.
/// Returns (min_distance, index_of_closest_plane).
fn min_signed_distance(planes: &[(f64, f64, f64, f64)], point: &[f64; 3]) -> (f64, usize) {
    let mut min_d: f64 = f64::MAX;
    let mut min_i: usize = 0;
    for (i, &(nx, ny, nz, d)) in planes.iter().enumerate() {
        let dist: f64 = nx * point[0] + ny * point[1] + nz * point[2] + d;
        if dist < min_d {
            min_d = dist;
            min_i = i;
        }
    }
    (min_d, min_i)
}

/// Collect inward-facing plane equations from triangular faces.
/// Each plane is (nx, ny, nz, d) where nx*x + ny*y + nz*z + d = signed distance.
fn collect_face_planes(input: &PolyData) -> Vec<(f64, f64, f64, f64)> {
    let n = input.points.len();
    if n == 0 {
        return Vec::new();
    }

    // Centroid for orienting normals inward
    let mut cx: f64 = 0.0;
    let mut cy: f64 = 0.0;
    let mut cz: f64 = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;

    let mut planes = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = input.points.get(cell[0] as usize);
        let b = input.points.get(cell[1] as usize);
        let c = input.points.get(cell[2] as usize);

        // Cross product (b-a) x (c-a)
        let abx: f64 = b[0] - a[0];
        let aby: f64 = b[1] - a[1];
        let abz: f64 = b[2] - a[2];
        let acx: f64 = c[0] - a[0];
        let acy: f64 = c[1] - a[1];
        let acz: f64 = c[2] - a[2];

        let mut nx: f64 = aby * acz - abz * acy;
        let mut ny: f64 = abz * acx - abx * acz;
        let mut nz: f64 = abx * acy - aby * acx;
        let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-15 {
            continue;
        }
        nx /= len;
        ny /= len;
        nz /= len;

        // d such that nx*x + ny*y + nz*z + d = 0 on the plane
        let d: f64 = -(nx * a[0] + ny * a[1] + nz * a[2]);

        // Ensure normal points inward (toward centroid)
        let dist_to_centroid: f64 = nx * cx + ny * cy + nz * cz + d;
        if dist_to_centroid < 0.0 {
            planes.push((-nx, -ny, -nz, -d));
        } else {
            planes.push((nx, ny, nz, d));
        }
    }

    planes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_cube() {
        // Build a unit cube centered at origin from [-0.5, 0.5]^3
        let mut pd = PolyData::new();
        pd.points.push([-0.5, -0.5, -0.5]);
        pd.points.push([0.5, -0.5, -0.5]);
        pd.points.push([0.5, 0.5, -0.5]);
        pd.points.push([-0.5, 0.5, -0.5]);
        pd.points.push([-0.5, -0.5, 0.5]);
        pd.points.push([0.5, -0.5, 0.5]);
        pd.points.push([0.5, 0.5, 0.5]);
        pd.points.push([-0.5, 0.5, 0.5]);

        // 12 triangles (2 per face)
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.polys.push_cell(&[4, 6, 5]);
        pd.polys.push_cell(&[4, 7, 6]);
        pd.polys.push_cell(&[0, 4, 5]);
        pd.polys.push_cell(&[0, 5, 1]);
        pd.polys.push_cell(&[2, 6, 7]);
        pd.polys.push_cell(&[2, 7, 3]);
        pd.polys.push_cell(&[0, 3, 7]);
        pd.polys.push_cell(&[0, 7, 4]);
        pd.polys.push_cell(&[1, 5, 6]);
        pd.polys.push_cell(&[1, 6, 2]);

        let (radius, center) = inscribed_sphere(&pd);
        // Inscribed sphere of unit cube has radius 0.5
        assert!((radius - 0.5).abs() < 0.05, "radius={}", radius);
        assert!(center[0].abs() < 0.05, "cx={}", center[0]);
        assert!(center[1].abs() < 0.05, "cy={}", center[1]);
        assert!(center[2].abs() < 0.05, "cz={}", center[2]);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let (radius, center) = inscribed_sphere(&pd);
        assert_eq!(radius, 0.0);
        assert_eq!(center, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn tetrahedron() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 1.0, 1.0]);
        pd.points.push([-1.0, -1.0, 1.0]);
        pd.points.push([-1.0, 1.0, -1.0]);
        pd.points.push([1.0, -1.0, -1.0]);

        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.polys.push_cell(&[1, 3, 2]);

        let (radius, _center) = inscribed_sphere(&pd);
        // Inscribed sphere of regular tetrahedron with edge 2*sqrt(2): r = edge/(2*sqrt(6)) ≈ 0.5774
        let expected: f64 = 2.0 * 2.0f64.sqrt() / (2.0 * 6.0f64.sqrt());
        assert!(
            (radius - expected).abs() < 0.1,
            "radius={}, expected={}",
            radius,
            expected
        );
    }
}
