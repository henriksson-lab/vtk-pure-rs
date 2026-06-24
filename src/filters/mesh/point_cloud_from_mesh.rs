use crate::data::{CellArray, Points, PolyData};

/// Sample random points on a triangle mesh surface.
///
/// Points are distributed proportional to triangle face area.
/// Returns a vertex-only PolyData (each point is a single vertex cell).
/// Uses a simple linear congruential generator seeded with `seed`.
pub fn random_sample_on_surface(input: &PolyData, num_points: usize, seed: u64) -> PolyData {
    if num_points == 0 {
        return PolyData::new();
    }

    // Collect triangles and compute areas
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    let mut areas: Vec<f64> = Vec::new();
    let mut total_area: f64 = 0.0;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        // Triangulate fan for polygons with more than 3 vertices
        let v0: usize = cell[0] as usize;
        for k in 1..cell.len() - 1 {
            let v1: usize = cell[k] as usize;
            let v2: usize = cell[k + 1] as usize;
            let p0 = input.points.get(v0);
            let p1 = input.points.get(v1);
            let p2 = input.points.get(v2);

            let e1: [f64; 3] = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2: [f64; 3] = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cross: [f64; 3] = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let area: f64 =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();

            triangles.push([v0, v1, v2]);
            areas.push(area);
            total_area += area;
        }
    }

    if triangles.is_empty() || total_area < 1e-30 {
        return PolyData::new();
    }

    // Build CDF
    let mut cdf: Vec<f64> = Vec::with_capacity(areas.len());
    let mut cumulative: f64 = 0.0;
    for a in &areas {
        cumulative += a / total_area;
        cdf.push(cumulative);
    }
    // Ensure last entry is exactly 1.0
    if let Some(last) = cdf.last_mut() {
        *last = 1.0;
    }

    // Simple LCG PRNG
    let mut rng_state: u64 = seed;
    let mut next_rand = || -> f64 {
        rng_state = rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Use upper bits for better quality
        let val: u64 = (rng_state >> 11) & 0x1FFFFFFFFFFFFF; // 53 bits
        val as f64 / (1u64 << 53) as f64
    };

    let mut points: Points<f64> = Points::new();
    let mut verts = CellArray::new();

    for _ in 0..num_points {
        // Pick a triangle proportional to area
        let r: f64 = next_rand();
        let tri_idx: usize = match cdf.binary_search_by(|v| v.partial_cmp(&r).unwrap()) {
            Ok(i) => i,
            Err(i) => i.min(cdf.len() - 1),
        };

        let [v0, v1, v2] = triangles[tri_idx];
        let p0 = input.points.get(v0);
        let p1 = input.points.get(v1);
        let p2 = input.points.get(v2);

        // Random point in triangle using barycentric coordinates
        let mut s: f64 = next_rand();
        let mut t: f64 = next_rand();
        if s + t > 1.0 {
            s = 1.0 - s;
            t = 1.0 - t;
        }
        let u: f64 = 1.0 - s - t;

        let pt: [f64; 3] = [
            u * p0[0] + s * p1[0] + t * p2[0],
            u * p0[1] + s * p1[1] + t * p2[1],
            u * p0[2] + s * p1[2] + t * p2[2],
        ];

        let idx: i64 = points.len() as i64;
        points.push(pt);
        verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.verts = verts;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_unit_triangle() -> PolyData {
        PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn correct_point_count() {
        let pd = make_unit_triangle();
        let result = random_sample_on_surface(&pd, 100, 42);
        assert_eq!(result.points.len(), 100);
        assert_eq!(result.verts.num_cells(), 100);
    }

    #[test]
    fn points_inside_triangle() {
        let pd = make_unit_triangle();
        let result = random_sample_on_surface(&pd, 200, 123);
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            // All points should be in the z=0 plane
            assert!(p[2].abs() < 1e-10);
            // x >= 0, y >= 0, x+y <= 1 (within triangle)
            assert!(p[0] > -1e-10);
            assert!(p[1] > -1e-10);
            assert!(p[0] + p[1] < 1.0 + 1e-10);
        }
    }

    #[test]
    fn deterministic_with_same_seed() {
        let pd = make_unit_triangle();
        let r1 = random_sample_on_surface(&pd, 50, 999);
        let r2 = random_sample_on_surface(&pd, 50, 999);
        for i in 0..50 {
            let a = r1.points.get(i);
            let b = r2.points.get(i);
            assert!((a[0] - b[0]).abs() < 1e-15);
            assert!((a[1] - b[1]).abs() < 1e-15);
            assert!((a[2] - b[2]).abs() < 1e-15);
        }
    }
}
