//! Project mesh points onto a height map using bilinear interpolation.
//!
//! For each mesh point (x, y), samples the 2D height map at that location
//! and sets z = sampled_height.

use crate::data::{ImageData, PolyData};

/// Project mesh points onto a 2D height map.
///
/// The height map is a 2D `ImageData` (nz = 1) with a scalar array
/// representing height values. For each mesh point, the (x, y) position
/// is used to sample the height map with bilinear interpolation, and
/// the point's z coordinate is set to the sampled height.
///
/// Points outside the height map bounds are clamped to the nearest edge.
pub fn fit_to_heightmap(mesh: &PolyData, heightmap: &ImageData) -> PolyData {
    let mut output = mesh.clone();
    let dims = heightmap.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    if mesh.points.is_empty() || mesh.total_cells() < 1 || nx < 2 || ny < 2 || nz != 1 {
        return output;
    }

    let origin = heightmap.origin();
    let spacing = heightmap.spacing();

    // Pre-read all height values
    let total = nx * ny;
    let mut heights = vec![0.0f64; total];
    if let Some(scalars) = heightmap.point_data().scalars() {
        let mut buf = [0.0f64];
        for i in 0..total {
            scalars.tuple_as_f64(i, &mut buf);
            heights[i] = buf[0];
        }
    } else {
        return output; // no scalars, nothing to do
    }

    for i in 0..output.points.len() {
        let p = output.points.get(i);
        let fi = (p[0] - origin[0]) / spacing[0];
        let fj = (p[1] - origin[1]) / spacing[1];

        let (i0, tx) = if fi < 0.0 {
            (0, 0.0)
        } else if fi >= (nx - 1) as f64 {
            (nx - 2, 1.0)
        } else {
            let i0 = fi.floor() as usize;
            (i0, fi - i0 as f64)
        };
        let (j0, ty) = if fj < 0.0 {
            (0, 0.0)
        } else if fj >= (ny - 1) as f64 {
            (ny - 2, 1.0)
        } else {
            let j0 = fj.floor() as usize;
            (j0, fj - j0 as f64)
        };
        let i1 = i0 + 1;
        let j1 = j0 + 1;

        // Bilinear interpolation
        let h00 = heights[j0 * nx + i0];
        let h10 = heights[j0 * nx + i1];
        let h01 = heights[j1 * nx + i0];
        let h11 = heights[j1 * nx + i1];

        let z = h00 * (1.0 - tx) * (1.0 - ty)
            + h10 * tx * (1.0 - ty)
            + h01 * (1.0 - tx) * ty
            + h11 * tx * ty;

        output.points.set(i, [p[0], p[1], z + origin[2]]);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn project_onto_flat_heightmap() {
        // Create a 3x3 height map with z = 5.0 everywhere
        let mut hm = ImageData::with_dimensions(3, 3, 1);
        hm.set_spacing([1.0, 1.0, 1.0]);
        hm.set_origin([0.0, 0.0, 0.0]);
        let heights = vec![5.0f64; 9];
        hm.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("height", heights, 1)));
        hm.point_data_mut().set_active_scalars("height");

        // Create a mesh with points at z = 0
        let pd = PolyData::from_triangles(
            vec![[0.5, 0.5, 0.0], [1.5, 0.5, 0.0], [1.0, 1.5, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = fit_to_heightmap(&pd, &hm);

        // All z values should be 5.0
        for i in 0..3 {
            let p = result.points.get(i);
            assert!((p[2] - 5.0).abs() < 1e-10, "point {} z = {}", i, p[2]);
        }
    }
}
