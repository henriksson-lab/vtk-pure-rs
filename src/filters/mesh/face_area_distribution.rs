//! Face area distribution analysis and area-based operations.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Compute face area histogram.
pub fn face_area_histogram(mesh: &PolyData, n_bins: usize) -> Table {
    let areas = compute_areas(mesh);
    if areas.is_empty() {
        return Table::new();
    }
    let min_a = areas.iter().cloned().fold(f64::MAX, f64::min);
    let max_a = areas.iter().cloned().fold(0.0f64, f64::max);
    let range = (max_a - min_a).max(1e-15);
    let bw = range / n_bins as f64;
    let mut counts = vec![0usize; n_bins];
    for &a in &areas {
        let bin = ((a - min_a) / bw) as usize;
        counts[bin.min(n_bins - 1)] += 1;
    }
    let centers: Vec<f64> = (0..n_bins).map(|i| min_a + (i as f64 + 0.5) * bw).collect();
    let count_f: Vec<f64> = counts.iter().map(|&c| c as f64).collect();
    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "AreaCenter",
            centers,
            1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Count", count_f, 1)))
}

/// Add per-face area as cell data.
pub fn add_face_areas(mesh: &PolyData) -> PolyData {
    let areas = compute_areas(mesh);
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("FaceArea", areas, 1)));
    result
}

/// Compute total surface area.
pub fn total_surface_area(mesh: &PolyData) -> f64 {
    compute_areas(mesh).iter().sum()
}

/// Compute area statistics: (min, max, mean, std, total).
pub fn area_statistics(mesh: &PolyData) -> (f64, f64, f64, f64, f64) {
    let areas = compute_areas(mesh);
    if areas.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let total: f64 = areas.iter().sum();
    let n = areas.len() as f64;
    let mean = total / n;
    let var = areas.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / n;
    let min = areas.iter().cloned().fold(f64::MAX, f64::min);
    let max = areas.iter().cloned().fold(0.0f64, f64::max);
    (min, max, mean, var.sqrt(), total)
}

fn compute_areas(mesh: &PolyData) -> Vec<f64> {
    mesh.polys
        .iter()
        .map(|cell| {
            if cell.len() < 3 {
                return 0.0;
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn areas() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (min, max, mean, _, total) = area_statistics(&mesh);
        assert!((total - 0.5).abs() < 0.01);
        assert!((min - 0.5).abs() < 0.01);
    }
    #[test]
    fn histogram() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let table = face_area_histogram(&mesh, 5);
        assert_eq!(table.num_rows(), 5);
    }
    #[test]
    fn add_areas() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = add_face_areas(&mesh);
        assert!(result.cell_data().get_array("FaceArea").is_some());
    }
}
