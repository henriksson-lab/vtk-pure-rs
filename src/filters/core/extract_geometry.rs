//! Extract geometry from any dataset type to PolyData.
//!
//! Unified conversion of ImageData, UnstructuredGrid, RectilinearGrid,
//! and StructuredGrid to their surface PolyData representation.

use crate::data::*;

/// Extract the surface geometry of any dataset as PolyData.
///
/// Dispatches to the appropriate conversion based on input type.
pub fn extract_geometry_image(image: &ImageData) -> PolyData {
    crate::filters::core::convert::image_data_surface_to_poly_data(image)
}

/// Extract surface from UnstructuredGrid.
pub fn extract_geometry_unstructured(grid: &UnstructuredGrid) -> PolyData {
    crate::filters::core::convert::unstructured_grid_to_poly_data(grid)
}

/// Extract surface from RectilinearGrid.
pub fn extract_geometry_rectilinear(grid: &RectilinearGrid) -> PolyData {
    crate::filters::core::convert::rectilinear_grid_to_poly_data(grid)
}

/// Extract surface from StructuredGrid.
pub fn extract_geometry_structured(grid: &StructuredGrid) -> PolyData {
    crate::filters::core::convert::structured_grid_to_poly_data(grid)
}

/// Extract geometry from a Block.
pub fn extract_geometry_block(block: &Block) -> Option<PolyData> {
    match block {
        Block::PolyData(pd) => Some(pd.clone()),
        Block::ImageData(id) => Some(extract_geometry_image(id)),
        Block::UnstructuredGrid(ug) => Some(extract_geometry_unstructured(ug)),
        Block::RectilinearGrid(rg) => Some(extract_geometry_rectilinear(rg)),
        Block::StructuredGrid(sg) => Some(extract_geometry_structured(sg)),
        Block::MultiBlock(mb) => {
            // Merge all blocks
            let mut meshes = Vec::new();
            for i in 0..mb.num_blocks() {
                if let Some(block) = mb.block(i) {
                    if let Some(pd) = extract_geometry_block(block) {
                        meshes.push(pd);
                    }
                }
            }
            if meshes.is_empty() {
                return None;
            }
            let refs: Vec<&PolyData> = meshes.iter().collect();
            Some(crate::filters::core::append::append(&refs))
        }
    }
}

/// Extract geometry from a MultiBlockDataSet (merges all blocks).
pub fn extract_geometry_multi_block(mb: &MultiBlockDataSet) -> PolyData {
    extract_geometry_block(&Block::MultiBlock(mb.clone())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_to_surface() {
        let image = ImageData::with_dimensions(5, 5, 5);
        let surface = extract_geometry_image(&image);
        assert!(surface.points.len() > 0);
        assert!(surface.polys.num_cells() > 0);
    }

    #[test]
    fn rectilinear_to_surface() {
        let grid =
            RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0, 1.0], vec![0.0, 1.0]);
        let surface = extract_geometry_rectilinear(&grid);
        assert!(surface.points.len() > 0);
    }

    #[test]
    fn multi_block_merge() {
        let mut mb = MultiBlockDataSet::new();
        mb.add_block(
            "a",
            Block::PolyData(PolyData::from_triangles(
                vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                vec![[0, 1, 2]],
            )),
        );
        mb.add_block(
            "b",
            Block::PolyData(PolyData::from_triangles(
                vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
                vec![[0, 1, 2]],
            )),
        );

        let merged = extract_geometry_multi_block(&mb);
        assert_eq!(merged.points.len(), 6);
        assert_eq!(merged.polys.num_cells(), 2);
    }

    #[test]
    fn block_dispatch() {
        let block = Block::ImageData(ImageData::with_dimensions(3, 3, 3));
        let pd = extract_geometry_block(&block).unwrap();
        assert!(pd.points.len() > 0);
    }
}
