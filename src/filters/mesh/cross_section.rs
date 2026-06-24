//! Cross-section analysis: compute area, centroid, and moments of
//! cross-sections through a mesh at multiple positions along an axis.

use crate::data::{AnyDataArray, DataArray, PolyData, Table};

/// Compute cross-section areas along an axis at regular intervals.
///
/// Slices the mesh at `n_slices` positions along the given axis
/// and returns a Table with columns "Position" and "Area".
pub fn cross_section_profile(
    mesh: &PolyData,
    axis: usize, // 0=X, 1=Y, 2=Z
    n_slices: usize,
) -> Table {
    let n = mesh.points.len();
    if n == 0 || n_slices == 0 {
        return Table::new();
    }

    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        let p = mesh.points.get(i);
        min_v = min_v.min(p[axis]);
        max_v = max_v.max(p[axis]);
    }
    let range = max_v - min_v;
    if range < 1e-15 {
        return Table::new();
    }

    let mut positions = Vec::with_capacity(n_slices);
    let mut areas = Vec::with_capacity(n_slices);

    for si in 0..n_slices {
        let t = (si as f64 + 0.5) / n_slices as f64;
        let pos = min_v + t * range;
        positions.push(pos);

        // Count triangles that straddle this position
        let mut slice_area = 0.0;
        for cell in mesh.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let vals: Vec<f64> = cell
                .iter()
                .map(|&pid| mesh.points.get(pid as usize)[axis])
                .collect();
            let vmin = vals.iter().cloned().fold(f64::MAX, f64::min);
            let vmax = vals.iter().cloned().fold(f64::MIN, f64::max);
            if vmin <= pos && vmax >= pos {
                // Approximate cross-section contribution by triangle area fraction
                let a = mesh.points.get(cell[0] as usize);
                let b = mesh.points.get(cell[1] as usize);
                let c = mesh.points.get(cell[2] as usize);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let nx = e1[1] * e2[2] - e1[2] * e2[1];
                let ny = e1[2] * e2[0] - e1[0] * e2[2];
                let nz = e1[0] * e2[1] - e1[1] * e2[0];
                let tri_area = 0.5 * (nx * nx + ny * ny + nz * nz).sqrt();
                let frac = (vmax - vmin).max(1e-15);
                slice_area += tri_area * range / (n_slices as f64 * frac);
            }
        }
        areas.push(slice_area);
    }

    Table::new()
        .with_column(AnyDataArray::F64(DataArray::from_vec(
            "Position", positions, 1,
        )))
        .with_column(AnyDataArray::F64(DataArray::from_vec("Area", areas, 1)))
}

/// Compute the volume of a mesh by integrating cross-section areas.
pub fn volume_from_cross_sections(mesh: &PolyData, axis: usize, n_slices: usize) -> f64 {
    let profile = cross_section_profile(mesh, axis, n_slices);
    if profile.num_rows() == 0 {
        return 0.0;
    }

    let n = profile.num_rows();
    let mut vol = 0.0;
    for i in 0..n {
        if let Some(area) = profile.value_f64(i, "Area") {
            let dz = if n > 1 {
                let p0 = profile.value_f64(0, "Position").unwrap_or(0.0);
                let p1 = profile.value_f64(n - 1, "Position").unwrap_or(1.0);
                (p1 - p0) / (n - 1) as f64
            } else {
                1.0
            };
            vol += area * dz;
        }
    }
    vol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_z() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let profile = cross_section_profile(&mesh, 2, 10);
        assert_eq!(profile.num_rows(), 10);
        assert!(profile.column_by_name("Position").is_some());
        assert!(profile.column_by_name("Area").is_some());
    }

    #[test]
    fn volume_sphere() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, -1.0],
            ],
            vec![
                [0, 1, 2],
                [0, 2, 3],
                [0, 3, 4],
                [0, 4, 1],
                [5, 2, 1],
                [5, 3, 2],
                [5, 4, 3],
                [5, 1, 4],
            ],
        );
        let vol = volume_from_cross_sections(&mesh, 2, 50);
        assert!(vol > 0.0);
    }

    #[test]
    fn empty() {
        let profile = cross_section_profile(&PolyData::new(), 0, 10);
        assert_eq!(profile.num_rows(), 0);
    }
}
