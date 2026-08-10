//! Mesh deformation energy metrics: Dirichlet, area distortion, angle distortion.

use crate::data::{AnyDataArray, DataArray, PolyData};

pub use crate::filters::mesh::laplacian_energy::dirichlet_energy;

/// Compute per-triangle area distortion between two meshes.
///
/// Returns the ratio of each triangle's area in `deformed` vs `original`.
///
/// Thin wrapper over [`crate::filters::mesh::mesh_area_distortion::area_distortion`],
/// which computes the same quantity in logarithmic form (and takes the measured
/// mesh first); this entry point exponentiates it back to a plain ratio.
pub fn area_distortion(original: &PolyData, deformed: &PolyData) -> PolyData {
    let mut result =
        crate::filters::mesh::mesh_area_distortion::area_distortion(deformed, original);

    let ratios: Option<Vec<f64>> = result.cell_data().get_array("AreaDistortion").map(|arr| {
        let mut buf = [0.0f64];
        (0..arr.num_tuples())
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0].exp()
            })
            .collect()
    });

    if let Some(ratios) = ratios {
        result
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "AreaDistortion",
                ratios,
                1,
            )));
    }
    result
}

/// Compute per-triangle angle distortion between two meshes.
///
/// Measures the maximum angle change per triangle.
///
/// Thin wrapper over [`crate::filters::mesh::conformal_factor::angle_distortion`],
/// which takes the measured mesh first.
pub fn angle_distortion(original: &PolyData, deformed: &PolyData) -> PolyData {
    crate::filters::mesh::conformal_factor::angle_distortion(deformed, original)
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
    fn stretch() {
        let orig = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((stretch_metric(&orig, &orig)).abs() < 1e-10);
    }
}
