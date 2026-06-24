//! Adaptive Mesh Refinement (AMR) dataset.
//!
//! An AMR dataset consists of multiple refinement levels, each containing
//! one or more `ImageData` blocks at that level's resolution.

use crate::data::ImageData;
use crate::types::VtkError;

/// A single refinement level in an AMR hierarchy.
#[derive(Debug, Clone)]
pub struct AMRLevel {
    /// Level index (0 = coarsest).
    pub level: usize,
    /// Blocks at this refinement level.
    pub blocks: Vec<ImageData>,
    /// Grid spacing for this level.
    pub spacing: [f64; 3],
}

/// Adaptive Mesh Refinement dataset with multiple resolution levels.
///
/// Level 0 is the coarsest, and higher levels have finer spacing.
#[derive(Debug, Clone, Default)]
pub struct AMRDataSet {
    /// Refinement levels from coarsest to finest.
    pub levels: Vec<AMRLevel>,
}

impl AMRDataSet {
    /// Create an empty AMR dataset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new refinement level with the given spacing. Returns the level index.
    pub fn add_level(&mut self, spacing: [f64; 3]) -> usize {
        validate_spacing(spacing).expect("AMR level spacing must be finite and positive");
        let level = self.levels.len();
        self.levels.push(AMRLevel {
            level,
            blocks: Vec::new(),
            spacing,
        });
        level
    }

    /// Add a block to the given level. Returns the block index within that level.
    pub fn add_block(&mut self, level: usize, block: ImageData) -> usize {
        self.try_add_block(level, block)
            .expect("AMR block level and metadata must be valid")
    }

    /// Try to add a block to the given level.
    pub fn try_add_block(&mut self, level: usize, block: ImageData) -> Result<usize, VtkError> {
        let amr_level = self
            .levels
            .get(level)
            .ok_or_else(|| VtkError::index_oob(level, self.levels.len()))?;
        validate_block_spacing(amr_level, &block)?;
        let idx = amr_level.blocks.len();
        self.levels[level].blocks.push(block);
        Ok(idx)
    }

    /// Number of refinement levels.
    pub fn num_levels(&self) -> usize {
        self.levels.len()
    }

    /// Number of blocks at the given level.
    pub fn num_blocks(&self, level: usize) -> usize {
        self.levels.get(level).map_or(0, |l| l.blocks.len())
    }

    /// Get a reference to a specific block.
    pub fn block(&self, level: usize, idx: usize) -> Option<&ImageData> {
        self.levels.get(level).and_then(|l| l.blocks.get(idx))
    }

    /// Get a refinement level by index.
    pub fn level(&self, level: usize) -> Option<&AMRLevel> {
        self.levels.get(level)
    }

    /// Total number of blocks across all levels.
    pub fn total_blocks(&self) -> usize {
        self.levels.iter().map(|l| l.blocks.len()).sum()
    }

    /// Spacing of the coarsest (level 0) level.
    ///
    /// Returns `[0.0, 0.0, 0.0]` if there are no levels.
    pub fn coarsest_spacing(&self) -> [f64; 3] {
        self.levels.first().map_or([0.0; 3], |l| l.spacing)
    }

    /// Spacing of the finest (highest) level.
    ///
    /// Returns `[0.0, 0.0, 0.0]` if there are no levels.
    pub fn finest_spacing(&self) -> [f64; 3] {
        self.levels.last().map_or([0.0; 3], |l| l.spacing)
    }

    /// Validate level numbering and block metadata consistency.
    pub fn validate(&self) -> Result<(), VtkError> {
        for (idx, level) in self.levels.iter().enumerate() {
            if level.level != idx {
                return Err(VtkError::InvalidData(format!(
                    "AMR level metadata mismatch: stored {}, expected {}",
                    level.level, idx
                )));
            }
            validate_spacing(level.spacing)?;
            for block in &level.blocks {
                validate_block_spacing(level, block)?;
            }
        }
        Ok(())
    }
}

fn validate_spacing(spacing: [f64; 3]) -> Result<(), VtkError> {
    if spacing.iter().all(|v| v.is_finite() && *v > 0.0) {
        Ok(())
    } else {
        Err(VtkError::InvalidData(format!(
            "AMR spacing must be finite and positive, got {spacing:?}"
        )))
    }
}

fn validate_block_spacing(level: &AMRLevel, block: &ImageData) -> Result<(), VtkError> {
    let block_spacing = block.spacing();
    if block_spacing
        .iter()
        .zip(level.spacing)
        .all(|(a, b)| (*a - b).abs() <= f64::EPSILON)
    {
        Ok(())
    } else {
        Err(VtkError::InvalidData(format!(
            "AMR block spacing {:?} does not match level {} spacing {:?}",
            block_spacing, level.level, level.spacing
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amr_basic() {
        let mut amr = AMRDataSet::new();
        let l0 = amr.add_level([1.0, 1.0, 1.0]);
        let l1 = amr.add_level([0.5, 0.5, 0.5]);

        let mut block0 = ImageData::with_dimensions(10, 10, 1);
        block0.set_spacing([1.0, 1.0, 1.0]);
        amr.add_block(l0, block0);

        let mut block1 = ImageData::with_dimensions(20, 20, 1);
        block1.set_spacing([0.5, 0.5, 0.5]);
        amr.add_block(l1, block1);

        assert_eq!(amr.num_levels(), 2);
        assert_eq!(amr.num_blocks(0), 1);
        assert_eq!(amr.num_blocks(1), 1);
        assert_eq!(amr.total_blocks(), 2);
        assert_eq!(amr.coarsest_spacing(), [1.0, 1.0, 1.0]);
        assert_eq!(amr.finest_spacing(), [0.5, 0.5, 0.5]);
    }

    #[test]
    fn amr_multiple_blocks() {
        let mut amr = AMRDataSet::new();
        let l0 = amr.add_level([2.0, 2.0, 2.0]);
        for _ in 0..3 {
            let mut block = ImageData::with_dimensions(5, 5, 5);
            block.set_spacing([2.0, 2.0, 2.0]);
            amr.add_block(l0, block);
        }

        assert_eq!(amr.num_blocks(0), 3);
        assert_eq!(amr.total_blocks(), 3);
        assert_eq!(amr.block(0, 1).unwrap().dimensions(), [5, 5, 5]);
    }

    #[test]
    fn checked_access_rejects_invalid_indices() {
        let amr = AMRDataSet::new();
        assert_eq!(amr.num_blocks(0), 0);
        assert!(amr.block(0, 0).is_none());
    }

    #[test]
    fn try_add_block_rejects_bad_level_and_spacing() {
        let mut amr = AMRDataSet::new();
        let mut block = ImageData::with_dimensions(2, 2, 2);
        block.set_spacing([1.0, 1.0, 1.0]);
        assert!(amr.try_add_block(0, block.clone()).is_err());

        let level = amr.add_level([0.5, 0.5, 0.5]);
        assert!(amr.try_add_block(level, block).is_err());
    }

    #[test]
    fn validate_catches_level_metadata_mismatch() {
        let mut amr = AMRDataSet::new();
        amr.add_level([1.0, 1.0, 1.0]);
        amr.levels[0].level = 99;
        assert!(amr.validate().is_err());
    }
}
