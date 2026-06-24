//! Resample data onto an AMR / HyperTreeGrid hierarchy.
//!
//! Samples a uniform ImageData onto the coarse cells of a HyperTreeGrid.

use crate::data::{AnyDataArray, DataArray, HyperTreeGrid, ImageData};

/// Resample an ImageData onto the coarse grid of a HyperTreeGrid.
///
/// For each coarse cell, averages all ImageData voxels that fall within
/// the cell bounds. Returns cell data values as a flat array indexed
/// by coarse cell linear index.
pub fn amr_resample(htg: &HyperTreeGrid, source: &ImageData, array_name: &str) -> Vec<f64> {
    let gs = htg.grid_size();
    let bounds = htg.bounds();
    let htg_spacing = [
        (bounds.x_max - bounds.x_min) / gs[0] as f64,
        (bounds.y_max - bounds.y_min) / gs[1] as f64,
        if gs[2] > 1 {
            (bounds.z_max - bounds.z_min) / gs[2] as f64
        } else {
            1.0
        },
    ];
    let htg_origin = [bounds.x_min, bounds.y_min, bounds.z_min];

    let src_dims = source.dimensions();
    let src_spacing = source.spacing();
    let src_origin = source.origin();

    let arr = match source.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return vec![0.0; gs[0] * gs[1] * gs[2]],
    };

    let n_coarse = gs[0] * gs[1] * gs[2];
    let mut values = vec![0.0f64; n_coarse];

    let mut buf = [0.0f64];

    for k in 0..gs[2] {
        for j in 0..gs[1] {
            for i in 0..gs[0] {
                let cell_min = [
                    htg_origin[0] + i as f64 * htg_spacing[0],
                    htg_origin[1] + j as f64 * htg_spacing[1],
                    htg_origin[2] + k as f64 * htg_spacing[2],
                ];
                let cell_max = [
                    cell_min[0] + htg_spacing[0],
                    cell_min[1] + htg_spacing[1],
                    cell_min[2] + htg_spacing[2],
                ];

                // Find overlapping voxels in source
                let ix_start = ((cell_min[0] - src_origin[0]) / src_spacing[0])
                    .floor()
                    .max(0.0) as usize;
                let ix_end = ((cell_max[0] - src_origin[0]) / src_spacing[0])
                    .ceil()
                    .min(src_dims[0] as f64) as usize;
                let iy_start = ((cell_min[1] - src_origin[1]) / src_spacing[1])
                    .floor()
                    .max(0.0) as usize;
                let iy_end = ((cell_max[1] - src_origin[1]) / src_spacing[1])
                    .ceil()
                    .min(src_dims[1] as f64) as usize;
                let iz_start = ((cell_min[2] - src_origin[2]) / src_spacing[2])
                    .floor()
                    .max(0.0) as usize;
                let iz_end = ((cell_max[2] - src_origin[2]) / src_spacing[2])
                    .ceil()
                    .min(src_dims[2] as f64) as usize;

                let mut sum = 0.0;
                let mut count = 0;
                for iz in iz_start..iz_end {
                    for iy in iy_start..iy_end {
                        for ix in ix_start..ix_end {
                            let idx = ix + iy * src_dims[0] + iz * src_dims[0] * src_dims[1];
                            if idx < arr.num_tuples() {
                                arr.tuple_as_f64(idx, &mut buf);
                                sum += buf[0];
                                count += 1;
                            }
                        }
                    }
                }

                let ci = i + j * gs[0] + k * gs[0] * gs[1];
                values[ci] = if count > 0 { sum / count as f64 } else { 0.0 };
            }
        }
    }

    values
}

/// Resample and store result as cell data on the HyperTreeGrid.
pub fn amr_resample_to_htg(htg: &mut HyperTreeGrid, source: &ImageData, array_name: &str) {
    let values = amr_resample(htg, source, array_name);
    htg.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_uniform() {
        let htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let source = ImageData::from_function(
            [10, 10, 1],
            [0.2, 0.2, 1.0],
            [0.0, 0.0, 0.0],
            "density",
            |x, y, _z| x + y,
        );
        let values = amr_resample(&htg, &source, "density");
        assert_eq!(values.len(), 4);
        // Bottom-left cell should have lower average than top-right
        assert!(values[0] < values[3]);
    }

    #[test]
    fn resample_3d() {
        let htg = HyperTreeGrid::new([2, 2, 2], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let source = ImageData::from_function(
            [4, 4, 4],
            [0.5, 0.5, 0.5],
            [0.0, 0.0, 0.0],
            "val",
            |x, _y, _z| x,
        );
        let values = amr_resample(&htg, &source, "val");
        assert_eq!(values.len(), 8);
    }

    #[test]
    fn missing_array() {
        let htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let source = ImageData::with_dimensions(5, 5, 1);
        let values = amr_resample(&htg, &source, "nonexistent");
        assert!(values.iter().all(|&v| v == 0.0));
    }
}
