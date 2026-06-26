//! Filters for HyperTreeGrid: geometry extraction, threshold, cell centers.
//!
//! Since the internal tree structure is private, these filters work through
//! the public API, primarily operating on the cell data arrays.

use crate::data::{AnyDataArray, DataArray, HyperTreeGrid, ImageData, Points, PolyData};

/// Extract the surface geometry of a HyperTreeGrid as quads on the coarse grid.
///
/// Creates one quad per visible coarse-grid face on the boundary.
pub fn hyper_tree_grid_geometry(htg: &HyperTreeGrid) -> PolyData {
    let gs = htg.grid_size();
    let bounds = htg.grid_bounds();
    let spacing = [
        (bounds.x_max - bounds.x_min) / gs[0] as f64,
        (bounds.y_max - bounds.y_min) / gs[1] as f64,
        if gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / gs[2] as f64
        } else {
            0.0
        },
    ];
    let origin = [bounds.x_min, bounds.y_min, bounds.z_min];

    let mut points = Points::<f64>::new();
    let mut polys = crate::data::CellArray::new();
    let mut point_map: std::collections::HashMap<[i64; 3], usize> =
        std::collections::HashMap::new();

    if htg.dimensions()[2] <= 1 {
        // 2D: one quad per coarse cell
        for j in 0..gs[1] {
            for i in 0..gs[0] {
                let x0 = origin[0] + i as f64 * spacing[0];
                let y0 = origin[1] + j as f64 * spacing[1];
                let x1 = x0 + spacing[0];
                let y1 = y0 + spacing[1];

                let corners = [[x0, y0, 0.0], [x1, y0, 0.0], [x1, y1, 0.0], [x0, y1, 0.0]];
                let mut ids = Vec::new();
                for c in &corners {
                    let key = [(c[0] * 1e6) as i64, (c[1] * 1e6) as i64, 0];
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
    } else {
        // 3D: boundary faces of the coarse grid
        for k in 0..gs[2] {
            for j in 0..gs[1] {
                for i in 0..gs[0] {
                    let x0 = origin[0] + i as f64 * spacing[0];
                    let y0 = origin[1] + j as f64 * spacing[1];
                    let z0 = origin[2] + k as f64 * spacing[2];
                    let x1 = x0 + spacing[0];
                    let y1 = y0 + spacing[1];
                    let z1 = z0 + spacing[2];

                    // Add boundary faces
                    let faces: Vec<[[f64; 3]; 4]> = {
                        let mut f = Vec::new();
                        if i == 0 {
                            f.push([[x0, y0, z0], [x0, y1, z0], [x0, y1, z1], [x0, y0, z1]]);
                        }
                        if i == gs[0] - 1 {
                            f.push([[x1, y0, z0], [x1, y0, z1], [x1, y1, z1], [x1, y1, z0]]);
                        }
                        if j == 0 {
                            f.push([[x0, y0, z0], [x0, y0, z1], [x1, y0, z1], [x1, y0, z0]]);
                        }
                        if j == gs[1] - 1 {
                            f.push([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]]);
                        }
                        if k == 0 {
                            f.push([[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]]);
                        }
                        if k == gs[2] - 1 {
                            f.push([[x0, y0, z1], [x0, y1, z1], [x1, y1, z1], [x1, y0, z1]]);
                        }
                        f
                    };

                    for face in &faces {
                        let mut ids = Vec::new();
                        for c in face {
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
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Extract cell centers of all coarse cells as points.
pub fn hyper_tree_grid_cell_centers(htg: &HyperTreeGrid) -> PolyData {
    let gs = htg.grid_size();
    let bounds = htg.grid_bounds();
    let spacing = [
        (bounds.x_max - bounds.x_min) / gs[0] as f64,
        (bounds.y_max - bounds.y_min) / gs[1] as f64,
        if gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / gs[2] as f64
        } else {
            0.0
        },
    ];
    let origin = [bounds.x_min, bounds.y_min, bounds.z_min];

    let mut points = Points::<f64>::new();
    for k in 0..gs[2] {
        for j in 0..gs[1] {
            for i in 0..gs[0] {
                let cx = origin[0] + (i as f64 + 0.5) * spacing[0];
                let cy = origin[1] + (j as f64 + 0.5) * spacing[1];
                let cz = if gs[2] > 1 {
                    origin[2] + (k as f64 + 0.5) * spacing[2]
                } else {
                    0.0
                };
                points.push([cx, cy, cz]);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;

    // Add depth info
    let depth_data = vec![0.0f64; gs[0] * gs[1] * gs[2]];
    // depth_data stays 0 for coarse cells (depth info would need tree traversal)
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CoarseDepth",
            depth_data,
            1,
        )));

    mesh
}

/// Convert a HyperTreeGrid to a uniform ImageData at the coarse level.
pub fn hyper_tree_grid_to_image_data(htg: &HyperTreeGrid) -> ImageData {
    let dims = htg.dimensions();
    let gs = htg.grid_size();
    let bounds = htg.grid_bounds();
    let spacing = [
        (bounds.x_max - bounds.x_min) / gs[0] as f64,
        (bounds.y_max - bounds.y_min) / gs[1] as f64,
        if gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / gs[2] as f64
        } else {
            1.0
        },
    ];

    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(spacing)
        .with_origin([bounds.x_min, bounds.y_min, bounds.z_min])
}

/// Get a summary of the HyperTreeGrid structure.
pub fn hyper_tree_grid_info(htg: &HyperTreeGrid) -> String {
    format!(
        "HyperTreeGrid: {}D, coarse grid {}x{}x{}, {} trees, {} total cells, max depth {}",
        htg.dimension(),
        htg.grid_size()[0],
        htg.grid_size()[1],
        htg.grid_size()[2],
        htg.num_trees(),
        htg.num_cells(),
        htg.max_depth(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_2d() {
        let htg = HyperTreeGrid::new([3, 3, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let geom = hyper_tree_grid_geometry(&htg);
        assert_eq!(geom.polys.num_cells(), 4); // one quad per coarse cell
        assert!(geom.points.len() > 0);
    }

    #[test]
    fn geometry_3d() {
        let htg = HyperTreeGrid::new([2, 2, 2], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let geom = hyper_tree_grid_geometry(&htg);
        assert!(geom.polys.num_cells() > 0);
    }

    #[test]
    fn cell_centers() {
        let htg = HyperTreeGrid::new([3, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let centers = hyper_tree_grid_cell_centers(&htg);
        assert_eq!(centers.points.len(), 6); // 2*3

        let p0 = centers.points.get(0);
        assert!((p0[0] - 0.5).abs() < 1e-10);
        assert!((p0[1] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn to_image_data() {
        let htg = HyperTreeGrid::new([5, 5, 5], [1.0, 2.0, 3.0], [0.5, 0.5, 0.5]);
        let img = hyper_tree_grid_to_image_data(&htg);
        assert_eq!(img.dimensions(), [5, 5, 5]);
        assert_eq!(img.origin(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn geometry_empty_grid_uses_grid_bounds() {
        let htg = HyperTreeGrid::new([3, 2, 1], [10.0, 20.0, 0.0], [2.0, 3.0, 1.0]);
        assert!(htg.bounds().is_empty());
        let geom = hyper_tree_grid_geometry(&htg);
        assert_eq!(geom.polys.num_cells(), 2);
        assert_eq!(geom.points.get(0), [10.0, 20.0, 0.0]);
    }

    #[test]
    fn info() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        htg.init_tree(0, 0, 0);
        htg.subdivide(0, 0, 0, 0);
        let s = hyper_tree_grid_info(&htg);
        assert!(s.contains("2D"));
        assert!(s.contains("trees"));
    }
}
