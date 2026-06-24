//! Contour on HyperTreeGrid coarse cell data.
//!
//! Extracts contour lines/surfaces at scalar isovalues on the coarse grid.

use crate::data::{CellArray, HyperTreeGrid, Points, PolyData};

/// Extract contour at an isovalue from HyperTreeGrid cell data.
///
/// Operates on the coarse grid level: for each pair of adjacent coarse cells
/// where the scalar crosses the isovalue, generates a contour face.
pub fn hyper_tree_grid_contour(htg: &HyperTreeGrid, array_name: &str, isovalue: f64) -> PolyData {
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

    let arr = match htg.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

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
                let _x = origin[0] + (i as f64 + 1.0 + t - 0.5) * spacing[0]; // approximate
                let x = origin[0] + (i + 1) as f64 * spacing[0]; // exact interface
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

                let y = origin[1] + (j + 1) as f64 * spacing[1];
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

                    let z = origin[2] + (k + 1) as f64 * spacing[2];
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

    #[test]
    fn contour_2d() {
        let mut htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let vals: Vec<f64> = (0..16).map(|i| i as f64).collect();
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("temp", vals, 1)));
        let contour = hyper_tree_grid_contour(&htg, "temp", 5.5);
        assert!(contour.polys.num_cells() > 0);
    }

    #[test]
    fn no_crossing() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let vals = vec![1.0, 2.0, 3.0, 4.0];
        htg.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 1)));
        let contour = hyper_tree_grid_contour(&htg, "v", 100.0);
        assert_eq!(contour.polys.num_cells(), 0);
    }

    #[test]
    fn missing_array() {
        let htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let contour = hyper_tree_grid_contour(&htg, "none", 0.5);
        assert_eq!(contour.polys.num_cells(), 0);
    }
}
