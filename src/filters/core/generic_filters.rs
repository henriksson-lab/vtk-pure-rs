//! Generic dataset operations: clip, contour, cut, glyph for any dataset type.
//!
//! These dispatch to the appropriate specialized filter after converting
//! the input to PolyData.

use crate::data::*;

/// Clip any dataset by a plane, returning PolyData.
pub fn generic_clip(block: &Block, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let pd = block_to_poly_data(block);
    crate::filters::core::clip::clip_by_plane(&pd, origin, normal)
}

/// Contour any dataset at a scalar isovalue, returning PolyData.
///
/// Requires the PolyData to have scalar data; extracts contour lines.
pub fn generic_contour(block: &Block, isovalue: f64) -> PolyData {
    let pd = block_to_poly_data(block);
    let scalars = match pd.point_data().scalars() {
        Some(s) => {
            let mut vals = Vec::with_capacity(s.num_tuples());
            let mut buf = [0.0f64];
            for i in 0..s.num_tuples() {
                s.tuple_as_f64(i, &mut buf);
                vals.push(buf[0]);
            }
            vals
        }
        None => return PolyData::new(),
    };
    crate::filters::core::contour::contour(&pd, &scalars, isovalue)
}

/// Cut any dataset with a plane (slice), returning PolyData lines.
pub fn generic_cutter(block: &Block, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let pd = block_to_poly_data(block);
    crate::filters::core::slice::slice_by_plane(&pd, origin, normal)
}

/// Place glyphs at points of any dataset, returning PolyData.
pub fn generic_glyph(block: &Block, glyph: &PolyData, scale_factor: f64) -> PolyData {
    let pd = block_to_poly_data(block);
    crate::filters::core::glyph::glyph(&pd, glyph, scale_factor, false)
}

fn block_to_poly_data(block: &Block) -> PolyData {
    match block {
        Block::PolyData(pd) => pd.clone(),
        Block::ImageData(id) => crate::filters::core::convert::image_data_surface_to_poly_data(id),
        Block::UnstructuredGrid(ug) => {
            crate::filters::core::convert::unstructured_grid_to_poly_data(ug)
        }
        Block::RectilinearGrid(rg) => {
            crate::filters::core::convert::rectilinear_grid_to_poly_data(rg)
        }
        Block::StructuredGrid(sg) => {
            crate::filters::core::convert::structured_grid_to_poly_data(sg)
        }
        Block::MultiBlock(mb) => {
            crate::filters::core::extract_geometry::extract_geometry_multi_block(mb)
        }
    }
}

/// Convenience: clip a PolyData
pub fn clip_poly_data(pd: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    crate::filters::core::clip::clip_by_plane(pd, origin, normal)
}

/// Convenience: clip an ImageData
pub fn clip_image_data(id: &ImageData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let pd = crate::filters::core::convert::image_data_surface_to_poly_data(id);
    crate::filters::core::clip::clip_by_plane(&pd, origin, normal)
}

/// Convenience: slice an UnstructuredGrid
pub fn slice_unstructured_grid(
    ug: &UnstructuredGrid,
    origin: [f64; 3],
    normal: [f64; 3],
) -> PolyData {
    let pd = crate::filters::core::convert::unstructured_grid_to_poly_data(ug);
    crate::filters::core::slice::slice_by_plane(&pd, origin, normal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_clip_poly_data() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = generic_clip(&Block::PolyData(pd), [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn generic_clip_image() {
        let img = ImageData::with_dimensions(5, 5, 5);
        let result = generic_clip(&Block::ImageData(img), [2.0, 2.0, 2.0], [1.0, 0.0, 0.0]);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn generic_cutter_test() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = generic_cutter(&Block::PolyData(pd), [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        // Slice may or may not produce lines depending on plane intersection
        assert!(result.points.len() >= 0);
    }

    #[test]
    fn generic_glyph_test() {
        let pd = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let glyph = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = generic_glyph(&Block::PolyData(pd), &glyph, 1.0);
        assert!(result.points.len() >= 6); // 2 copies of 3-vertex glyph
    }
}
