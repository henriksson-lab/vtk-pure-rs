use crate::data::PolyData;

/// Compute a histogram of triangle aspect ratios.
///
/// Returns (bin_centers, counts) with `n_bins` bins from 1.0 to max_ratio.
pub fn aspect_ratio_histogram(input: &PolyData, n_bins: usize) -> (Vec<f64>, Vec<usize>) {
    let mut ratios = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let d01 = dist(v0, v1);
            let d12 = dist(v1, v2);
            let d20 = dist(v2, v0);
            ratios.push(tri_aspect_ratio(d01, d12, d20, tri_area(v0, v1, v2)));
        }
    }

    if ratios.is_empty() {
        return (vec![], vec![]);
    }
    let nb = n_bins.max(1);
    let max_r = ratios
        .iter()
        .filter(|&&r| r < f64::MAX)
        .copied()
        .fold(1.0f64, f64::max);
    let bw = (max_r - 1.0).max(0.01) / nb as f64;

    let centers: Vec<f64> = (0..nb).map(|i| 1.0 + (i as f64 + 0.5) * bw).collect();
    let mut counts = vec![0usize; nb];
    for &r in &ratios {
        if r >= f64::MAX {
            continue;
        }
        let bin = ((r - 1.0) / bw).floor() as usize;
        counts[bin.min(nb - 1)] += 1;
    }
    (centers, counts)
}

/// Compute a histogram of triangle areas.
pub fn area_histogram(input: &PolyData, n_bins: usize) -> (Vec<f64>, Vec<usize>) {
    let mut areas = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            areas.push(tri_area(v0, v1, v2));
        }
    }

    if areas.is_empty() {
        return (vec![], vec![]);
    }
    let nb = n_bins.max(1);
    let max_a = areas.iter().copied().fold(0.0f64, f64::max);
    let bw = max_a.max(1e-15) / nb as f64;

    let centers: Vec<f64> = (0..nb).map(|i| (i as f64 + 0.5) * bw).collect();
    let mut counts = vec![0usize; nb];
    for &a in &areas {
        let bin = (a / bw).floor() as usize;
        counts[bin.min(nb - 1)] += 1;
    }
    (centers, counts)
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
fn tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}
fn tri_aspect_ratio(e0: f64, e1: f64, e2: f64, area: f64) -> f64 {
    if area <= 1e-15 {
        return f64::MAX;
    }
    let longest = e0.max(e1).max(e2);
    let perimeter = e0 + e1 + e2;
    longest * perimeter / (4.0 * 3.0f64.sqrt() * area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ar_histogram() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.01, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let (centers, counts) = aspect_ratio_histogram(&pd, 5);
        assert_eq!(centers.len(), 5);
        let total: usize = counts.iter().sum();
        assert_eq!(total, 2);
    }

    #[test]
    fn area_hist() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (centers, counts) = area_histogram(&pd, 3);
        let total: usize = counts.iter().sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn histogram_uses_vtk_triangle_aspect_ratio() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (centers, counts) = aspect_ratio_histogram(&pd, 1);
        let expected = (2.0f64.sqrt() * (2.0 + 2.0f64.sqrt())) / (2.0 * 3.0f64.sqrt());
        assert_eq!(counts, [1]);
        assert!((centers[0] - ((1.0 + expected) / 2.0)).abs() < 1e-12);
    }

    #[test]
    fn aspect_histogram_uses_all_polygon_fan_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let (_, counts) = aspect_ratio_histogram(&pd, 3);
        assert_eq!(counts.iter().sum::<usize>(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (c, _) = aspect_ratio_histogram(&pd, 5);
        assert!(c.is_empty());
    }
}
