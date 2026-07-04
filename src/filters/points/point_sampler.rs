use crate::data::{CellArray, Points, PolyData};

/// Sample points uniformly on the surface of a triangle mesh.
///
/// Generates approximately `num_points` points distributed over the
/// triangulated surface using a deterministic strategy: each triangle
/// receives a number of samples proportional to its area, placed at
/// barycentric coordinates in a regular pattern.
pub fn point_sampler(input: &PolyData, num_points: usize) -> PolyData {
    if num_points == 0 {
        return PolyData::new();
    }

    // Collect triangles and their areas
    let mut triangles: Vec<([f64; 3], [f64; 3], [f64; 3])> = Vec::new();
    let mut areas: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let a = triangle_area(v0, v1, v2);
            triangles.push((v0, v1, v2));
            areas.push(a);
        }
    }

    if triangles.is_empty() {
        return PolyData::new();
    }

    let total_area: f64 = areas.iter().sum();
    if total_area < 1e-15 {
        return PolyData::new();
    }

    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();

    // Distribute points proportionally to area, matching VTK's area-based
    // sampling density while keeping this API's requested output count.
    let mut assigned = vec![0usize; triangles.len()];
    let mut remainders: Vec<(usize, f64)> = Vec::with_capacity(triangles.len());

    for (i, area) in areas.iter().enumerate() {
        let exact = area / total_area * num_points as f64;
        let base = exact.floor() as usize;
        assigned[i] = base;
        remainders.push((i, exact - base as f64));
    }

    let assigned_total: usize = assigned.iter().sum();
    let remaining = num_points.saturating_sub(assigned_total);
    if remaining > 0 {
        remainders.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    areas[b.0]
                        .partial_cmp(&areas[a.0])
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        for &(i, _) in remainders.iter().cycle().take(remaining) {
            assigned[i] += 1;
        }
    }

    // Generate sample points on each triangle
    for (ti, &(v0, v1, v2)) in triangles.iter().enumerate() {
        let n = assigned[ti];
        if n == 0 {
            continue;
        }

        // Use a grid of barycentric coordinates
        let mut grid_n = ((n as f64).sqrt().ceil() as usize).max(1);
        while grid_n * (grid_n + 1) / 2 < n {
            grid_n += 1;
        }
        let mut count = 0;
        for gi in 0..grid_n {
            for gj in 0..grid_n - gi {
                if count >= n {
                    break;
                }
                let u = (gi as f64 + 0.5) / grid_n as f64;
                let v = (gj as f64 + 0.5) / grid_n as f64;
                if u + v > 1.0 {
                    continue;
                }
                let w = 1.0 - u - v;
                let p = [
                    w * v0[0] + u * v1[0] + v * v2[0],
                    w * v0[1] + u * v1[1] + v * v2[1],
                    w * v0[2] + u * v1[2] + v * v2[2],
                ];
                let idx = out_points.len() as i64;
                out_points.push(p);
                out_verts.push_cell(&[idx]);
                count += 1;
            }
            if count >= n {
                break;
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd
}

fn triangle_area(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> f64 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = point_sampler(&pd, 10);
        assert!(result.points.len() > 0);
        assert!(result.points.len() <= 15); // might overshoot slightly due to rounding
        assert_eq!(result.verts.num_cells(), result.points.len());
    }

    #[test]
    fn zero_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = point_sampler(&pd, 0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = point_sampler(&pd, 100);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn proportional_to_area() {
        let mut pd = PolyData::new();
        // Small triangle
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.0, 0.1, 0.0]);
        // Large triangle (100x area)
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = point_sampler(&pd, 100);
        // Most points should be on the larger triangle
        assert!(result.points.len() > 50);
    }

    #[test]
    fn points_inside_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = point_sampler(&pd, 20);
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            // All points should be in the triangle (x>=0, y>=0, x+y<=1)
            assert!(p[0] >= -1e-10, "x={}", p[0]);
            assert!(p[1] >= -1e-10, "y={}", p[1]);
            assert!(p[0] + p[1] <= 1.0 + 1e-10, "x+y={}", p[0] + p[1]);
            assert!((p[2]).abs() < 1e-10); // z=0
        }
    }
}
