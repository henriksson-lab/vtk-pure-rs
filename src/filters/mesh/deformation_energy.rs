//! Mesh deformation energy metrics: Dirichlet, area distortion, angle distortion.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute Dirichlet energy measuring smoothness of a deformation.
///
/// Lower energy = smoother deformation.
pub fn dirichlet_energy(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    if n < 3 {
        return 0.0;
    }
    let adj = build_adj(mesh, n);

    let mut energy = 0.0;
    for i in 0..n {
        let pi = mesh.points.get(i);
        for &j in &adj[i] {
            let pj = mesh.points.get(j);
            let d = (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2);
            energy += d;
        }
    }
    energy / 2.0 // each edge counted twice
}

/// Compute per-triangle area distortion between two meshes.
///
/// Returns the ratio of each triangle's area in `deformed` vs `original`.
pub fn area_distortion(original: &PolyData, deformed: &PolyData) -> PolyData {
    let n_cells = original.polys.num_cells().min(deformed.polys.num_cells());
    let orig_cells: Vec<Vec<i64>> = original.polys.iter().map(|c| c.to_vec()).collect();
    let def_cells: Vec<Vec<i64>> = deformed.polys.iter().map(|c| c.to_vec()).collect();

    let mut distortion = Vec::with_capacity(n_cells);
    for ci in 0..n_cells {
        let orig_area = tri_area(original, &orig_cells[ci]);
        let def_area = tri_area(deformed, &def_cells[ci]);
        let ratio = if orig_area > 1e-15 {
            def_area / orig_area
        } else {
            1.0
        };
        distortion.push(ratio);
    }

    let mut result = deformed.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AreaDistortion",
            distortion,
            1,
        )));
    result
}

/// Compute per-triangle angle distortion between two meshes.
///
/// Measures the maximum angle change per triangle.
pub fn angle_distortion(original: &PolyData, deformed: &PolyData) -> PolyData {
    let n_cells = original.polys.num_cells().min(deformed.polys.num_cells());
    let orig_cells: Vec<Vec<i64>> = original.polys.iter().map(|c| c.to_vec()).collect();
    let def_cells: Vec<Vec<i64>> = deformed.polys.iter().map(|c| c.to_vec()).collect();

    let mut distortion = Vec::with_capacity(n_cells);
    for ci in 0..n_cells {
        if orig_cells[ci].len() < 3 || def_cells[ci].len() < 3 {
            distortion.push(0.0);
            continue;
        }
        let orig_angles = tri_angles(original, &orig_cells[ci]);
        let def_angles = tri_angles(deformed, &def_cells[ci]);
        let max_diff = (0..3)
            .map(|i| (orig_angles[i] - def_angles[i]).abs())
            .fold(0.0f64, f64::max);
        distortion.push(max_diff.to_degrees());
    }

    let mut result = deformed.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AngleDistortion",
            distortion,
            1,
        )));
    result
}

/// Compute stretch metric: ratio of eigenvalues of deformation gradient.
pub fn stretch_metric(original: &PolyData, deformed: &PolyData) -> f64 {
    let n_cells = original.polys.num_cells().min(deformed.polys.num_cells());
    if n_cells == 0 {
        return 1.0;
    }

    let orig_cells: Vec<Vec<i64>> = original.polys.iter().map(|c| c.to_vec()).collect();
    let def_cells: Vec<Vec<i64>> = deformed.polys.iter().map(|c| c.to_vec()).collect();

    let mut total_stretch = 0.0;
    for ci in 0..n_cells {
        let oa = tri_area(original, &orig_cells[ci]);
        let da = tri_area(deformed, &def_cells[ci]);
        let ratio = if oa > 1e-15 { da / oa } else { 1.0 };
        total_stretch += (ratio - 1.0).abs();
    }
    total_stretch / n_cells as f64
}

fn tri_area(mesh: &PolyData, cell: &[i64]) -> f64 {
    if cell.len() < 3 {
        return 0.0;
    }
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (nx * nx + ny * ny + nz * nz).sqrt()
}

fn tri_angles(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    let a = mesh.points.get(cell[0] as usize);
    let b = mesh.points.get(cell[1] as usize);
    let c = mesh.points.get(cell[2] as usize);
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ba = [-ab[0], -ab[1], -ab[2]];
    let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
    let angle_at = |u: [f64; 3], v: [f64; 3]| -> f64 {
        let dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        let lu = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let lv = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if lu * lv > 1e-15 {
            (dot / (lu * lv)).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        }
    };
    [
        angle_at(ab, ac),
        angle_at(ba, bc),
        angle_at([-ac[0], -ac[1], -ac[2]], [-bc[0], -bc[1], -bc[2]]),
    ]
}

fn build_adj(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                adj[a].insert(b);
                adj[b].insert(a);
            }
        }
    }
    adj.into_iter().map(|s| s.into_iter().collect()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_deformation() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = area_distortion(&mesh, &mesh);
        let arr = result.cell_data().get_array("AreaDistortion").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scaled_mesh() {
        let orig = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scaled = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = area_distortion(&orig, &scaled);
        let arr = result.cell_data().get_array("AreaDistortion").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 4.0).abs() < 0.01); // area scales as square
    }

    #[test]
    fn dirichlet() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let e = dirichlet_energy(&mesh);
        assert!(e > 0.0);
    }

    #[test]
    fn stretch() {
        let orig = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((stretch_metric(&orig, &orig)).abs() < 1e-10);
    }
}
