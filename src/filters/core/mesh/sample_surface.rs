use crate::data::{CellArray, Points, PolyData};

/// Uniformly sample points on a triangle mesh surface.
///
/// Uses deterministic stratified sampling: divides each triangle's area
/// budget into a grid of barycentric coordinates. Produces exactly
/// `num_samples` points (approximately proportional to triangle area).
pub fn sample_surface_uniform(input: &PolyData, num_samples: usize) -> PolyData {
    if num_samples == 0 {
        return PolyData::new();
    }

    let mut tris: Vec<([f64; 3], [f64; 3], [f64; 3], f64)> = Vec::new();
    let mut total_area = 0.0;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if cell
            .iter()
            .any(|&id| id < 0 || id as usize >= input.points.len())
        {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let a = tri_area(v0, v1, v2);
            tris.push((v0, v1, v2, a));
            total_area += a;
        }
    }

    if tris.is_empty() || total_area < 1e-15 {
        return PolyData::new();
    }

    let mut out_points = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    let counts = allocate_samples_by_area(&tris, total_area, num_samples);

    for (&(v0, v1, v2, _), &n_for_tri) in tris.iter().zip(counts.iter()) {
        if n_for_tri == 0 {
            continue;
        }

        for k in 0..n_for_tri {
            let (u, v, w) = deterministic_barycentric(k, n_for_tri);
            let idx = out_points.len() as i64;
            out_points.push([
                w * v0[0] + u * v1[0] + v * v2[0],
                w * v0[1] + u * v1[1] + v * v2[1],
                w * v0[2] + u * v1[2] + v * v2[2],
            ]);
            out_verts.push_cell(&[idx]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.verts = out_verts;
    pd
}

fn allocate_samples_by_area(
    tris: &[([f64; 3], [f64; 3], [f64; 3], f64)],
    total_area: f64,
    num_samples: usize,
) -> Vec<usize> {
    let mut counts = Vec::with_capacity(tris.len());
    let mut remainders = Vec::with_capacity(tris.len());
    let mut assigned = 0usize;

    for (i, &(_, _, _, area)) in tris.iter().enumerate() {
        let exact = area / total_area * num_samples as f64;
        let count = exact.floor() as usize;
        counts.push(count);
        remainders.push((exact - count as f64, i));
        assigned += count;
    }

    remainders.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    for &(_, i) in remainders.iter().take(num_samples.saturating_sub(assigned)) {
        counts[i] += 1;
    }

    counts
}

fn deterministic_barycentric(k: usize, n: usize) -> (f64, f64, f64) {
    let r1 = (k as f64 + 0.5) / n as f64;
    let r2 = ((k as f64 + 0.5) * 0.618_033_988_749_894_8).fract();
    let sqrt_r1 = r1.sqrt();
    let u = 1.0 - sqrt_r1;
    let v = r2 * sqrt_r1;
    let w = 1.0 - u - v;
    (u, v, w)
}

fn tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_on_surface() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = sample_surface_uniform(&pd, 20);
        assert!(result.points.len() > 0);
        // All z should be 0 (triangle in XY plane)
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            assert!(p[2].abs() < 1e-10);
            assert!(p[0] >= -1e-10 && p[1] >= -1e-10);
        }
    }

    #[test]
    fn correct_count() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = sample_surface_uniform(&pd, 50);
        assert_eq!(result.points.len(), 50);
    }

    #[test]
    fn zero_samples() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = sample_surface_uniform(&pd, 0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = sample_surface_uniform(&pd, 100);
        assert_eq!(result.points.len(), 0);
    }
}
