use crate::data::PolyData;

/// Poisson disk sampling on a triangle mesh surface.
///
/// Generates well-distributed sample points on the surface with a minimum
/// distance guarantee between any two samples. Uses dart-throwing with a
/// simple pseudo-random generator seeded by `seed`.
///
/// Returns a vertex-only PolyData (points with vertex cells, no polys).
pub fn poisson_sample_surface(input: &PolyData, min_distance: f64, seed: u64) -> PolyData {
    let triangles = collect_triangles(input);
    if triangles.is_empty() || min_distance <= 0.0 {
        return PolyData::default();
    }

    // Compute per-triangle areas and total area
    let mut areas: Vec<f64> = Vec::with_capacity(triangles.len());
    let mut total_area: f64 = 0.0;
    for tri in &triangles {
        let a: f64 = triangle_area(tri);
        areas.push(a);
        total_area += a;
    }

    if total_area < 1e-30 {
        return PolyData::default();
    }

    // Build cumulative distribution for triangle selection
    let mut cdf: Vec<f64> = Vec::with_capacity(areas.len());
    let mut running: f64 = 0.0;
    for a in &areas {
        running += a / total_area;
        cdf.push(running);
    }

    // Estimate max number of samples
    let disk_area: f64 = std::f64::consts::PI * min_distance * min_distance / 4.0;
    let max_samples: usize = ((total_area / disk_area) * 4.0) as usize + 100;
    let max_attempts: usize = max_samples * 30;

    let mut rng_state: u64 = seed;
    let mut accepted: Vec<[f64; 3]> = Vec::new();

    for _ in 0..max_attempts {
        // Pick a random triangle weighted by area
        let r1: f64 = lcg_next_f64(&mut rng_state);
        let tri_idx: usize = match cdf.binary_search_by(|v| v.partial_cmp(&r1).unwrap()) {
            Ok(i) => i,
            Err(i) => i.min(triangles.len() - 1),
        };

        // Random point in triangle via barycentric coordinates
        let r2: f64 = lcg_next_f64(&mut rng_state);
        let r3: f64 = lcg_next_f64(&mut rng_state);
        let sqrt_r2: f64 = r2.sqrt();
        let u: f64 = 1.0 - sqrt_r2;
        let v: f64 = r3 * sqrt_r2;
        let w: f64 = 1.0 - u - v;

        let tri = &triangles[tri_idx];
        let px: f64 = u * tri[0][0] + v * tri[0][1] + w * tri[0][2];
        let py: f64 = u * tri[1][0] + v * tri[1][1] + w * tri[1][2];
        let pz: f64 = u * tri[2][0] + v * tri[2][1] + w * tri[2][2];
        let candidate: [f64; 3] = [px, py, pz];

        // Check minimum distance against all accepted points
        let mut too_close: bool = false;
        for p in &accepted {
            let dx: f64 = p[0] - candidate[0];
            let dy: f64 = p[1] - candidate[1];
            let dz: f64 = p[2] - candidate[2];
            let dist_sq: f64 = dx * dx + dy * dy + dz * dz;
            if dist_sq < min_distance * min_distance {
                too_close = true;
                break;
            }
        }

        if !too_close {
            accepted.push(candidate);
            if accepted.len() >= max_samples {
                break;
            }
        }
    }

    // Build vertex-only PolyData
    let mut pd = PolyData::default();
    for (i, pt) in accepted.iter().enumerate() {
        pd.points.push(*pt);
        pd.verts.push_cell(&[i as i64]);
    }
    pd
}

/// Collect triangles as [[x0,x1,x2],[y0,y1,y2],[z0,z1,z2]].
fn collect_triangles(input: &PolyData) -> Vec<[[f64; 3]; 3]> {
    let mut tris = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        // Fan-triangulate polygons
        let p0 = input.points.get(cell[0] as usize);
        for j in 1..cell.len() - 1 {
            let p1 = input.points.get(cell[j] as usize);
            let p2 = input.points.get(cell[j + 1] as usize);
            // Store as rows = coordinate axes: [xs, ys, zs]
            tris.push([
                [p0[0], p1[0], p2[0]],
                [p0[1], p1[1], p2[1]],
                [p0[2], p1[2], p2[2]],
            ]);
        }
    }
    tris
}

fn triangle_area(tri: &[[f64; 3]; 3]) -> f64 {
    let ax: f64 = tri[0][1] - tri[0][0];
    let ay: f64 = tri[1][1] - tri[1][0];
    let az: f64 = tri[2][1] - tri[2][0];
    let bx: f64 = tri[0][2] - tri[0][0];
    let by: f64 = tri[1][2] - tri[1][0];
    let bz: f64 = tri[2][2] - tri[2][0];
    let cx: f64 = ay * bz - az * by;
    let cy: f64 = az * bx - ax * bz;
    let cz: f64 = ax * by - ay * bx;
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

/// Simple LCG pseudo-random number generator returning [0, 1).
fn lcg_next_f64(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Take upper bits for better quality
    let val: u64 = (*state >> 11) & 0x001F_FFFF_FFFF_FFFF;
    val as f64 / (1u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad() -> PolyData {
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [10.0, 10.0, 0.0],
                [0.0, 10.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        )
    }

    #[test]
    fn generates_points_with_minimum_distance() {
        let pd = make_quad();
        let min_dist: f64 = 2.0;
        let result = poisson_sample_surface(&pd, min_dist, 42);
        assert!(
            result.points.len() > 0,
            "should generate at least one point"
        );

        // Verify minimum distance property
        let n: usize = result.points.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let a = result.points.get(i);
                let b = result.points.get(j);
                let dx: f64 = a[0] - b[0];
                let dy: f64 = a[1] - b[1];
                let dz: f64 = a[2] - b[2];
                let dist: f64 = (dx * dx + dy * dy + dz * dz).sqrt();
                assert!(
                    dist >= min_dist - 1e-10,
                    "points {} and {} are too close: {}",
                    i,
                    j,
                    dist
                );
            }
        }
    }

    #[test]
    fn points_lie_on_surface() {
        let pd = make_quad();
        let result = poisson_sample_surface(&pd, 1.0, 123);
        // All points should have z == 0 since the mesh is in the xy plane
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            assert!(
                p[2].abs() < 1e-10,
                "point {} has z = {}, expected 0",
                i,
                p[2]
            );
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::default();
        let result = poisson_sample_surface(&pd, 1.0, 0);
        assert_eq!(result.points.len(), 0);
    }
}
