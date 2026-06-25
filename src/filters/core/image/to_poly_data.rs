use crate::data::{CellArray, ImageData, Points, PolyData};

/// Convert the surface of an ImageData to a PolyData.
///
/// Generates quads for the 6 faces of the bounding box of the image grid.
/// Useful for visualizing the extent of an ImageData volume.
pub fn image_data_to_surface(image: &ImageData) -> PolyData {
    let dims = image.dimensions();
    if dims[0] < 2 || dims[1] < 2 || dims[2] < 2 {
        return PolyData::new();
    }

    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];

    let mut points = Points::new();
    let mut polys = CellArray::new();

    // Collect all unique surface points and generate quads for each face
    // Face ordering: -X, +X, -Y, +Y, -Z, +Z

    // Helper: add a grid face and return the base point index
    let add_face = |i_range: std::ops::Range<usize>,
                    j_range: std::ops::Range<usize>,
                    make_ijk: &dyn Fn(usize, usize) -> (usize, usize, usize),
                    pts: &mut Points<f64>,
                    polys: &mut CellArray| {
        let base = pts.len();
        let ni = i_range.len();
        let nj = j_range.len();

        for j in j_range.clone() {
            for i in i_range.clone() {
                let (ii, jj, kk) = make_ijk(i, j);
                pts.push(image.point_from_ijk(ii, jj, kk));
            }
        }

        for j in 0..nj - 1 {
            for i in 0..ni - 1 {
                let p0 = (base + j * ni + i) as i64;
                let p1 = p0 + 1;
                let p2 = p0 + ni as i64 + 1;
                let p3 = p0 + ni as i64;
                polys.push_cell(&[p0, p1, p2, p3]);
            }
        }
    };

    // -Z face (k=0)
    add_face(0..nx, 0..ny, &|i, j| (i, j, 0), &mut points, &mut polys);
    // +Z face (k=nz-1)
    add_face(
        0..nx,
        0..ny,
        &|i, j| (i, j, nz - 1),
        &mut points,
        &mut polys,
    );
    // -Y face (j=0)
    add_face(0..nx, 0..nz, &|i, k| (i, 0, k), &mut points, &mut polys);
    // +Y face (j=ny-1)
    add_face(
        0..nx,
        0..nz,
        &|i, k| (i, ny - 1, k),
        &mut points,
        &mut polys,
    );
    // -X face (i=0)
    add_face(0..ny, 0..nz, &|j, k| (0, j, k), &mut points, &mut polys);
    // +X face (i=nx-1)
    add_face(
        0..ny,
        0..nz,
        &|j, k| (nx - 1, j, k),
        &mut points,
        &mut polys,
    );

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_image_surface() {
        let image = ImageData::with_dimensions(3, 3, 3);
        let pd = image_data_to_surface(&image);
        // 6 faces, each with (3-1)*(3-1) = 4 quads = 24 total
        assert_eq!(pd.polys.num_cells(), 24);
        assert!(pd.points.len() > 0);
    }

    #[test]
    fn minimal_image() {
        let image = ImageData::with_dimensions(2, 2, 2);
        let pd = image_data_to_surface(&image);
        // 6 faces * 1 quad each = 6
        assert_eq!(pd.polys.num_cells(), 6);
    }

    #[test]
    fn too_small() {
        let image = ImageData::with_dimensions(1, 2, 2);
        let pd = image_data_to_surface(&image);
        assert_eq!(pd.polys.num_cells(), 0);
    }
}
