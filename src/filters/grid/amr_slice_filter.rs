//! Slice through AMR / HyperTreeGrid data.
//!
//! Extracts a 2D slice from the coarse grid at a given axis position.

use crate::data::{AnyDataArray, CellArray, DataArray, HyperTreeGrid, Points, PolyData};

/// Slice a HyperTreeGrid along an axis at a given position.
///
/// `axis`: 0=X, 1=Y, 2=Z. Returns a PolyData with quads at the slice plane.
pub fn amr_slice(htg: &HyperTreeGrid, axis: usize, position: f64) -> PolyData {
    let gs = htg.grid_size();
    let bounds = htg.bounds();
    let spacing = [
        (bounds.x_max - bounds.x_min) / gs[0] as f64,
        (bounds.y_max - bounds.y_min) / gs[1] as f64,
        if gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / gs[2] as f64
        } else {
            1.0
        },
    ];
    let origin = [bounds.x_min, bounds.y_min, bounds.z_min];

    // Find which coarse cell layer the slice passes through
    let cell_idx = ((position - origin[axis]) / spacing[axis]).floor() as usize;
    let max_idx = gs[axis].saturating_sub(1);
    let _cell_idx = cell_idx.min(max_idx);

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut point_map: std::collections::HashMap<[i64; 3], usize> =
        std::collections::HashMap::new();

    // Generate quads on the slice plane
    let (dim_a, dim_b) = match axis {
        0 => (1, 2), // YZ plane
        1 => (0, 2), // XZ plane
        _ => (0, 1), // XY plane
    };

    for ja in 0..gs[dim_a] {
        for jb in 0..gs[dim_b] {
            let mut corners = [[0.0f64; 3]; 4];
            for (ci, &(da, db)) in [(0, 0), (1, 0), (1, 1), (0, 1)].iter().enumerate() {
                let mut p = [position, position, position];
                p[dim_a] = origin[dim_a] + (ja + da) as f64 * spacing[dim_a];
                p[dim_b] = origin[dim_b] + (jb + db) as f64 * spacing[dim_b];
                p[axis] = position;
                corners[ci] = p;
            }

            let mut ids = Vec::new();
            for c in &corners {
                let key = [
                    (c[0] * 1e6) as i64,
                    (c[1] * 1e6) as i64,
                    (c[2] * 1e6) as i64,
                ];
                let idx = *point_map.entry(key).or_insert_with(|| {
                    let idx = points.len();
                    points.push(*c);
                    idx
                });
                ids.push(idx as i64);
            }
            polys.push_cell(&ids);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Resample a HyperTreeGrid onto a uniform 2D slice as ImageData scalars.
///
/// Returns a PolyData with the slice geometry and "SliceIndex" cell data.
pub fn amr_slice_with_indices(htg: &HyperTreeGrid, axis: usize, position: f64) -> PolyData {
    let mut mesh = amr_slice(htg, axis, position);
    let n_cells = mesh.polys.num_cells();
    let indices: Vec<f64> = (0..n_cells).map(|i| i as f64).collect();
    mesh.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SliceIndex",
            indices,
            1,
        )));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_2d_grid() {
        let htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = amr_slice(&htg, 2, 0.0); // XY slice at z=0
        assert_eq!(slice.polys.num_cells(), 16); // 4x4 cells
    }

    #[test]
    fn slice_3d_grid_x() {
        let htg = HyperTreeGrid::new([4, 3, 2], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = amr_slice(&htg, 0, 2.0); // YZ slice at x=2
        assert_eq!(slice.polys.num_cells(), 6); // 3*2
    }

    #[test]
    fn slice_3d_grid_y() {
        let htg = HyperTreeGrid::new([4, 3, 2], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = amr_slice(&htg, 1, 1.5); // XZ slice at y=1.5
        assert_eq!(slice.polys.num_cells(), 8); // 4*2
    }

    #[test]
    fn with_indices() {
        let htg = HyperTreeGrid::new([3, 3, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = amr_slice_with_indices(&htg, 2, 0.0);
        assert!(slice.cell_data().get_array("SliceIndex").is_some());
    }
}
