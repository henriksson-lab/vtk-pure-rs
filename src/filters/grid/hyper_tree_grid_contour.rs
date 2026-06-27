//! Contour on HyperTreeGrid coarse cell data.
//!
//! Extracts contour lines/surfaces at scalar isovalues on the coarse grid.

use crate::data::{CellArray, HyperTreeGrid, Points, PolyData};
use crate::types::BoundingBox;

/// Extract contour at an isovalue from HyperTreeGrid cell data.
///
/// Operates on the coarse grid level: for each pair of adjacent coarse cells
/// where the scalar crosses the isovalue, generates a contour face.
pub fn hyper_tree_grid_contour(htg: &HyperTreeGrid, array_name: &str, isovalue: f64) -> PolyData {
    let arr = match htg.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let leaves = htg.leaves();
    if !leaves.is_empty() {
        return contour_leaves(htg, &leaves, arr.as_ref(), isovalue);
    }

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
    let origin = [bounds.x_min, bounds.y_min, bounds.z_min];

    let cell_idx = |i: usize, j: usize, k: usize| -> usize { i + j * gs[0] + k * gs[0] * gs[1] };

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<[i64; 3], usize> = std::collections::HashMap::new();
    let mut buf = [0.0f64];

    // Check X-direction interfaces
    for k in 0..gs[2] {
        for j in 0..gs[1] {
            for i in 0..gs[0].saturating_sub(1) {
                let ci0 = cell_idx(i, j, k);
                let ci1 = cell_idx(i + 1, j, k);
                if ci0 >= arr.num_tuples() || ci1 >= arr.num_tuples() {
                    continue;
                }
                arr.tuple_as_f64(ci0, &mut buf);
                let v0 = buf[0];
                arr.tuple_as_f64(ci1, &mut buf);
                let v1 = buf[0];
                if (v0 - isovalue) * (v1 - isovalue) >= 0.0 {
                    continue;
                }

                let t = (isovalue - v0) / (v1 - v0);
                let x = origin[0] + (i as f64 + 0.5 + t) * spacing[0];
                let y0 = origin[1] + j as f64 * spacing[1];
                let y1 = y0 + spacing[1];
                let z0 = origin[2] + k as f64 * spacing[2];
                let z1 = z0 + spacing[2];

                let corners = [[x, y0, z0], [x, y1, z0], [x, y1, z1], [x, y0, z1]];
                add_quad(&mut points, &mut polys, &mut pt_map, &corners);
            }
        }
    }

    // Check Y-direction interfaces
    for k in 0..gs[2] {
        for j in 0..gs[1].saturating_sub(1) {
            for i in 0..gs[0] {
                let ci0 = cell_idx(i, j, k);
                let ci1 = cell_idx(i, j + 1, k);
                if ci0 >= arr.num_tuples() || ci1 >= arr.num_tuples() {
                    continue;
                }
                arr.tuple_as_f64(ci0, &mut buf);
                let v0 = buf[0];
                arr.tuple_as_f64(ci1, &mut buf);
                let v1 = buf[0];
                if (v0 - isovalue) * (v1 - isovalue) >= 0.0 {
                    continue;
                }

                let t = (isovalue - v0) / (v1 - v0);
                let y = origin[1] + (j as f64 + 0.5 + t) * spacing[1];
                let x0 = origin[0] + i as f64 * spacing[0];
                let x1 = x0 + spacing[0];
                let z0 = origin[2] + k as f64 * spacing[2];
                let z1 = z0 + spacing[2];

                let corners = [[x0, y, z0], [x1, y, z0], [x1, y, z1], [x0, y, z1]];
                add_quad(&mut points, &mut polys, &mut pt_map, &corners);
            }
        }
    }

    // Check Z-direction interfaces (for 3D)
    if gs[2] > 1 {
        for k in 0..gs[2].saturating_sub(1) {
            for j in 0..gs[1] {
                for i in 0..gs[0] {
                    let ci0 = cell_idx(i, j, k);
                    let ci1 = cell_idx(i, j, k + 1);
                    if ci0 >= arr.num_tuples() || ci1 >= arr.num_tuples() {
                        continue;
                    }
                    arr.tuple_as_f64(ci0, &mut buf);
                    let v0 = buf[0];
                    arr.tuple_as_f64(ci1, &mut buf);
                    let v1 = buf[0];
                    if (v0 - isovalue) * (v1 - isovalue) >= 0.0 {
                        continue;
                    }

                    let t = (isovalue - v0) / (v1 - v0);
                    let z = origin[2] + (k as f64 + 0.5 + t) * spacing[2];
                    let x0 = origin[0] + i as f64 * spacing[0];
                    let x1 = x0 + spacing[0];
                    let y0 = origin[1] + j as f64 * spacing[1];
                    let y1 = y0 + spacing[1];

                    let corners = [[x0, y0, z], [x1, y0, z], [x1, y1, z], [x0, y1, z]];
                    add_quad(&mut points, &mut polys, &mut pt_map, &corners);
                }
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

fn contour_leaves(
    htg: &HyperTreeGrid,
    leaves: &[crate::data::HyperTreeLeaf],
    arr: &dyn crate::data::DataArrayTrait,
    isovalue: f64,
) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<[i64; 3], usize> = std::collections::HashMap::new();
    let mut buf = [0.0f64];

    for i in 0..leaves.len() {
        if leaves[i].global_id >= arr.num_tuples() {
            continue;
        }
        arr.tuple_as_f64(leaves[i].global_id, &mut buf);
        let v0 = buf[0];

        for j in i + 1..leaves.len() {
            if leaves[j].global_id >= arr.num_tuples() {
                continue;
            }
            let Some((axis, overlap)) = shared_face(&leaves[i].bounds, &leaves[j].bounds) else {
                continue;
            };

            arr.tuple_as_f64(leaves[j].global_id, &mut buf);
            let v1 = buf[0];
            if (v0 - isovalue) * (v1 - isovalue) >= 0.0 {
                continue;
            }

            let c0 = leaves[i].bounds.center();
            let c1 = leaves[j].bounds.center();
            let denom = v1 - v0;
            if denom.abs() <= 1e-15 {
                continue;
            }
            let t = (isovalue - v0) / denom;
            let coord = c0[axis] + t * (c1[axis] - c0[axis]);
            let corners = contour_face_corners(htg.dimension(), axis, coord, overlap);
            add_quad(&mut points, &mut polys, &mut pt_map, &corners);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

fn shared_face(a: &BoundingBox, b: &BoundingBox) -> Option<(usize, [[f64; 2]; 3])> {
    const EPS: f64 = 1e-12;
    let mins_a = [a.x_min, a.y_min, a.z_min];
    let maxs_a = [a.x_max, a.y_max, a.z_max];
    let mins_b = [b.x_min, b.y_min, b.z_min];
    let maxs_b = [b.x_max, b.y_max, b.z_max];

    for axis in 0..3 {
        let touches = (maxs_a[axis] - mins_b[axis]).abs() <= EPS
            || (maxs_b[axis] - mins_a[axis]).abs() <= EPS;
        if !touches {
            continue;
        }

        let mut overlap = [[0.0; 2]; 3];
        let mut valid = true;
        for other in 0..3 {
            overlap[other] = [
                mins_a[other].max(mins_b[other]),
                maxs_a[other].min(maxs_b[other]),
            ];
            if other != axis && overlap[other][1] - overlap[other][0] <= EPS {
                valid = false;
                break;
            }
        }
        if valid {
            return Some((axis, overlap));
        }
    }
    None
}

fn contour_face_corners(
    dimension: usize,
    axis: usize,
    coord: f64,
    overlap: [[f64; 2]; 3],
) -> [[f64; 3]; 4] {
    let z0 = if dimension < 3 { 0.0 } else { overlap[2][0] };
    let z1 = if dimension < 3 { 1.0 } else { overlap[2][1] };
    match axis {
        0 => [
            [coord, overlap[1][0], z0],
            [coord, overlap[1][1], z0],
            [coord, overlap[1][1], z1],
            [coord, overlap[1][0], z1],
        ],
        1 => [
            [overlap[0][0], coord, z0],
            [overlap[0][1], coord, z0],
            [overlap[0][1], coord, z1],
            [overlap[0][0], coord, z1],
        ],
        _ => [
            [overlap[0][0], overlap[1][0], coord],
            [overlap[0][1], overlap[1][0], coord],
            [overlap[0][1], overlap[1][1], coord],
            [overlap[0][0], overlap[1][1], coord],
        ],
    }
}

fn add_quad(
    points: &mut Points<f64>,
    polys: &mut CellArray,
    pt_map: &mut std::collections::HashMap<[i64; 3], usize>,
    corners: &[[f64; 3]; 4],
) {
    let mut ids = Vec::new();
    for c in corners {
        let key = [
            (c[0] * 1e6) as i64,
            (c[1] * 1e6) as i64,
            (c[2] * 1e6) as i64,
        ];
        let idx = *pt_map.entry(key).or_insert_with(|| {
            let idx = points.len();
            points.push(*c);
            idx
        });
        ids.push(idx as i64);
    }
    polys.push_cell(&ids);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn contour_2d() {
        let mut htg = HyperTreeGrid::new([5, 5, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let vals: Vec<f64> = (0..16).map(|i| i as f64).collect();
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("temp", vals, 1)));
        let contour = hyper_tree_grid_contour(&htg, "temp", 5.5);
        assert!(contour.polys.num_cells() > 0);
    }

    #[test]
    fn contour_interpolates_between_cell_centers() {
        let mut htg = HyperTreeGrid::new([3, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 2.0],
                1,
            )));
        let contour = hyper_tree_grid_contour(&htg, "v", 0.5);
        assert_eq!(contour.polys.num_cells(), 1);
        assert!((contour.points.get(0)[0] - 0.75).abs() < 1e-12);
    }

    #[test]
    fn no_crossing() {
        let mut htg = HyperTreeGrid::new([3, 3, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let vals = vec![1.0, 2.0, 3.0, 4.0];
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 1)));
        let contour = hyper_tree_grid_contour(&htg, "v", 100.0);
        assert_eq!(contour.polys.num_cells(), 0);
    }

    #[test]
    fn missing_array() {
        let htg = HyperTreeGrid::new([3, 3, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let contour = hyper_tree_grid_contour(&htg, "none", 0.5);
        assert_eq!(contour.polys.num_cells(), 0);
    }
}
