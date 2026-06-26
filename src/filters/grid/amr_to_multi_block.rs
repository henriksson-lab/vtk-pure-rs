//! Convert AMR (HyperTreeGrid) data to MultiBlockDataSet.

use crate::data::*;

/// Convert a HyperTreeGrid to a MultiBlockDataSet where each coarse cell
/// becomes a separate ImageData block.
pub fn amr_to_multi_block(htg: &HyperTreeGrid) -> MultiBlockDataSet {
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

    let mut mb = MultiBlockDataSet::new();

    for k in 0..gs[2] {
        for j in 0..gs[1] {
            for i in 0..gs[0] {
                let origin = [
                    bounds.x_min + i as f64 * spacing[0],
                    bounds.y_min + j as f64 * spacing[1],
                    if gs[2] > 1 {
                        bounds.z_min + k as f64 * spacing[2]
                    } else {
                        0.0
                    },
                ];

                let block = ImageData::with_dimensions(2, 2, if gs[2] > 1 { 2 } else { 1 })
                    .with_spacing([spacing[0], spacing[1], spacing[2]])
                    .with_origin(origin);

                mb.add_block(format!("block_{i}_{j}_{k}"), Block::ImageData(block));
            }
        }
    }

    mb
}

/// Convert a MultiBlockDataSet of ImageData blocks back into a uniform ImageData.
///
/// Assumes all blocks have the same spacing and cover the domain uniformly.
pub fn multi_block_to_image(mb: &MultiBlockDataSet) -> Option<ImageData> {
    // Find global bounds
    let mut global_min = [f64::MAX; 3];
    let mut global_max = [f64::MIN; 3];
    let mut spacing = [1.0; 3];
    let mut found = false;

    for i in 0..mb.num_blocks() {
        if let Some(Block::ImageData(img)) = mb.block(i) {
            let o = img.origin();
            let s = img.spacing();
            let d = img.dimensions();
            spacing = s;
            found = true;
            for j in 0..3 {
                global_min[j] = global_min[j].min(o[j]);
                global_max[j] = global_max[j].max(o[j] + (d[j] as f64 - 1.0) * s[j]);
            }
        }
    }

    if !found {
        return None;
    }

    let dims = [
        ((global_max[0] - global_min[0]) / spacing[0]).round() as usize + 1,
        ((global_max[1] - global_min[1]) / spacing[1]).round() as usize + 1,
        ((global_max[2] - global_min[2]) / spacing[2]).round() as usize + 1,
    ];

    Some(
        ImageData::with_dimensions(dims[0], dims[1], dims[2])
            .with_spacing(spacing)
            .with_origin(global_min),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn htg_to_multiblock() {
        let htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let mb = amr_to_multi_block(&htg);
        assert_eq!(mb.num_blocks(), 9); // 3*3
    }

    #[test]
    fn htg_3d_to_multiblock() {
        let htg = HyperTreeGrid::new([3, 3, 3], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let mb = amr_to_multi_block(&htg);
        assert_eq!(mb.num_blocks(), 8);
    }

    #[test]
    fn multiblock_to_image() {
        let htg = HyperTreeGrid::new([3, 3, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let mb = amr_to_multi_block(&htg);
        let img = multi_block_to_image(&mb).unwrap();
        assert!(img.dimensions()[0] > 0);
    }

    #[test]
    fn empty_multiblock() {
        let mb = MultiBlockDataSet::new();
        assert!(multi_block_to_image(&mb).is_none());
    }
}
