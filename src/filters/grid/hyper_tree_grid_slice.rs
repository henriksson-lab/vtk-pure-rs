//! Slice through HyperTreeGrid at arbitrary plane orientations.
//!
//! Extends amr_slice_filter with support for oblique slicing planes.

use crate::data::{AnyDataArray, CellArray, DataArray, HyperTreeGrid, Points, PolyData};

/// Slice a HyperTreeGrid with an arbitrary plane defined by origin and normal.
///
/// For each coarse cell that the plane intersects, generates a polygon
/// at the intersection.
pub fn hyper_tree_grid_slice(
    htg: &HyperTreeGrid,
    plane_origin: [f64; 3],
    plane_normal: [f64; 3],
) -> PolyData {
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

    // Normalize plane normal
    let nlen = (plane_normal[0].powi(2) + plane_normal[1].powi(2) + plane_normal[2].powi(2)).sqrt();
    if nlen < 1e-15 {
        return PolyData::new();
    }
    let n = [
        plane_normal[0] / nlen,
        plane_normal[1] / nlen,
        plane_normal[2] / nlen,
    ];

    let mut all_points = Points::<f64>::new();
    let mut all_polys = CellArray::new();
    let mut cell_idx_data: Vec<f64> = Vec::new();

    for k in 0..gs[2] {
        for j in 0..gs[1] {
            for i in 0..gs[0] {
                let x0 = origin[0] + i as f64 * spacing[0];
                let y0 = origin[1] + j as f64 * spacing[1];
                let z0 = origin[2] + k as f64 * spacing[2];

                // 8 corners of the cell (or 4 for 2D)
                let corners: Vec<[f64; 3]> = if gs[2] <= 1 {
                    vec![
                        [x0, y0, 0.0],
                        [x0 + spacing[0], y0, 0.0],
                        [x0 + spacing[0], y0 + spacing[1], 0.0],
                        [x0, y0 + spacing[1], 0.0],
                    ]
                } else {
                    vec![
                        [x0, y0, z0],
                        [x0 + spacing[0], y0, z0],
                        [x0 + spacing[0], y0 + spacing[1], z0],
                        [x0, y0 + spacing[1], z0],
                        [x0, y0, z0 + spacing[2]],
                        [x0 + spacing[0], y0, z0 + spacing[2]],
                        [x0 + spacing[0], y0 + spacing[1], z0 + spacing[2]],
                        [x0, y0 + spacing[1], z0 + spacing[2]],
                    ]
                };

                // Signed distances from plane
                let dists: Vec<f64> = corners
                    .iter()
                    .map(|c| {
                        (c[0] - plane_origin[0]) * n[0]
                            + (c[1] - plane_origin[1]) * n[1]
                            + (c[2] - plane_origin[2]) * n[2]
                    })
                    .collect();

                // Check if plane intersects this cell
                let has_pos = dists.iter().any(|&d| d > 0.0);
                let has_neg = dists.iter().any(|&d| d < 0.0);
                if !has_pos || !has_neg {
                    continue;
                }

                // Find intersection points on edges
                let edges: Vec<(usize, usize)> = if gs[2] <= 1 {
                    vec![(0, 1), (1, 2), (2, 3), (3, 0)]
                } else {
                    vec![
                        (0, 1),
                        (1, 2),
                        (2, 3),
                        (3, 0), // bottom
                        (4, 5),
                        (5, 6),
                        (6, 7),
                        (7, 4), // top
                        (0, 4),
                        (1, 5),
                        (2, 6),
                        (3, 7), // verticals
                    ]
                };

                let mut intersection_pts: Vec<[f64; 3]> = Vec::new();
                for &(a, b) in &edges {
                    if (dists[a] > 0.0) != (dists[b] > 0.0) {
                        let t = dists[a] / (dists[a] - dists[b]);
                        let pt = [
                            corners[a][0] + t * (corners[b][0] - corners[a][0]),
                            corners[a][1] + t * (corners[b][1] - corners[a][1]),
                            corners[a][2] + t * (corners[b][2] - corners[a][2]),
                        ];
                        intersection_pts.push(pt);
                    }
                }

                if intersection_pts.len() >= 3 {
                    // Sort points by angle around centroid for proper polygon winding
                    let cx: f64 = intersection_pts.iter().map(|p| p[0]).sum::<f64>()
                        / intersection_pts.len() as f64;
                    let cy: f64 = intersection_pts.iter().map(|p| p[1]).sum::<f64>()
                        / intersection_pts.len() as f64;
                    let cz: f64 = intersection_pts.iter().map(|p| p[2]).sum::<f64>()
                        / intersection_pts.len() as f64;

                    // Build local 2D coordinate system on the plane
                    let up = if n[0].abs() < 0.9 {
                        [1.0, 0.0, 0.0]
                    } else {
                        [0.0, 1.0, 0.0]
                    };
                    let u = cross(n, up);
                    let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
                    let u = [u[0] / ul, u[1] / ul, u[2] / ul];
                    let v = cross(n, u);

                    intersection_pts.sort_by(|a, b| {
                        let da = [a[0] - cx, a[1] - cy, a[2] - cz];
                        let db = [b[0] - cx, b[1] - cy, b[2] - cz];
                        let angle_a = (da[0] * v[0] + da[1] * v[1] + da[2] * v[2])
                            .atan2(da[0] * u[0] + da[1] * u[1] + da[2] * u[2]);
                        let angle_b = (db[0] * v[0] + db[1] * v[1] + db[2] * v[2])
                            .atan2(db[0] * u[0] + db[1] * u[1] + db[2] * u[2]);
                        angle_a
                            .partial_cmp(&angle_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let _base_idx = all_points.len();
                    let mut ids = Vec::new();
                    for pt in &intersection_pts {
                        ids.push(all_points.len() as i64);
                        all_points.push(*pt);
                    }
                    all_polys.push_cell(&ids);
                    cell_idx_data.push((i + j * gs[0] + k * gs[0] * gs[1]) as f64);
                }
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = all_points;
    mesh.polys = all_polys;
    mesh.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CoarseCellIndex",
            cell_idx_data,
            1,
        )));
    mesh
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_aligned_slice() {
        let htg = HyperTreeGrid::new([4, 4, 4], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = hyper_tree_grid_slice(&htg, [2.5, 2.0, 2.0], [1.0, 0.0, 0.0]);
        assert!(slice.polys.num_cells() > 0);
    }

    #[test]
    fn diagonal_slice() {
        let htg = HyperTreeGrid::new([4, 4, 4], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = hyper_tree_grid_slice(&htg, [2.0, 2.0, 2.0], [1.0, 1.0, 0.0]);
        assert!(slice.polys.num_cells() > 0);
    }

    #[test]
    fn slice_outside() {
        let htg = HyperTreeGrid::new([2, 2, 2], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = hyper_tree_grid_slice(&htg, [10.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(slice.polys.num_cells(), 0);
    }

    #[test]
    fn has_cell_index() {
        let htg = HyperTreeGrid::new([3, 3, 3], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let slice = hyper_tree_grid_slice(&htg, [1.5, 1.5, 1.5], [0.0, 0.0, 1.0]);
        assert!(slice.cell_data().get_array("CoarseCellIndex").is_some());
    }
}
