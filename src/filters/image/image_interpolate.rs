//! Image interpolation methods (nearest and bicubic).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Resize image using nearest-neighbor interpolation.
pub fn resize_nearest(input: &ImageData, scalars: &str, new_nx: usize, new_ny: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (ox, oy, oz) = (dims[0], dims[1], dims[2]);
    if ox == 0 || oy == 0 || oz == 0 {
        return input.clone();
    }
    let (new_nx, new_ny) = (
        if new_nx > 0 { new_nx } else { ox },
        if new_ny > 0 { new_ny } else { oy },
    );
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.clone()
        })
        .collect();
    let data: Vec<f64> = (0..new_nx * new_ny * oz)
        .flat_map(|idx| {
            let iz = idx / (new_nx * new_ny);
            let plane_idx = idx % (new_nx * new_ny);
            let iy = plane_idx / new_nx;
            let ix = plane_idx % new_nx;
            let sx = if ox > 1 {
                (ix as f64 * (ox - 1) as f64 / (new_nx - 1).max(1) as f64).round() as usize
            } else {
                0
            };
            let sy = if oy > 1 {
                (iy as f64 * (oy - 1) as f64 / (new_ny - 1).max(1) as f64).round() as usize
            } else {
                0
            };
            let in_idx = (sx.min(ox - 1) + sy.min(oy - 1) * ox + iz * ox * oy) * num_components;
            vals[in_idx..in_idx + num_components].to_vec()
        })
        .collect();
    let (extent, origin, spacing) = resized_geometry(input, new_nx, new_ny, oz);
    let mut output = ImageData::with_dimensions(new_nx, new_ny, oz)
        .with_spacing(spacing)
        .with_origin(origin)
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            data,
            num_components,
        )));
    output.set_extent(extent);
    output
}

/// Resize image using bicubic interpolation.
pub fn resize_bicubic(input: &ImageData, scalars: &str, new_nx: usize, new_ny: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (ox, oy, oz) = (dims[0], dims[1], dims[2]);
    if ox == 0 || oy == 0 || oz == 0 {
        return input.clone();
    }
    let (new_nx, new_ny) = (
        if new_nx > 0 { new_nx } else { ox },
        if new_ny > 0 { new_ny } else { oy },
    );
    let num_components = arr.num_components();
    let mut buf = vec![0.0f64; num_components];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.clone()
        })
        .collect();
    let get = |x: isize, y: isize, z: usize, c: usize| -> f64 {
        let cx = x.clamp(0, ox as isize - 1) as usize;
        let cy = y.clamp(0, oy as isize - 1) as usize;
        vals[(cx + cy * ox + z * ox * oy) * num_components + c]
    };
    let data: Vec<f64> = (0..new_nx * new_ny * oz)
        .flat_map(|idx| {
            let iz = idx / (new_nx * new_ny);
            let plane_idx = idx % (new_nx * new_ny);
            let iy = plane_idx / new_nx;
            let ix = plane_idx % new_nx;
            let fx = ix as f64 * (ox - 1) as f64 / (new_nx - 1).max(1) as f64;
            let fy = iy as f64 * (oy - 1) as f64 / (new_ny - 1).max(1) as f64;
            let x0 = fx.floor() as isize;
            let y0 = fy.floor() as isize;
            let dx = fx - x0 as f64;
            let dy = fy - y0 as f64;
            let mut tuple = Vec::with_capacity(num_components);
            for c in 0..num_components {
                let mut sum = 0.0;
                for j in -1..=2isize {
                    let wy = cubic_weight(dy - j as f64);
                    for i in -1..=2isize {
                        let wx = cubic_weight(dx - i as f64);
                        sum += get(x0 + i, y0 + j, iz, c) * wx * wy;
                    }
                }
                tuple.push(sum);
            }
            tuple
        })
        .collect();
    let (extent, origin, spacing) = resized_geometry(input, new_nx, new_ny, oz);
    let mut output = ImageData::with_dimensions(new_nx, new_ny, oz)
        .with_spacing(spacing)
        .with_origin(origin)
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            data,
            num_components,
        )));
    output.set_extent(extent);
    output
}

/// Output extent, origin and spacing for a resize, following
/// `vtkImageResize::RequestInformation` in its `OUTPUT_DIMENSIONS` mode with
/// `Border` off: the low extent index is kept, the index stretch is
/// `(hi - lo) / (dims - 1)`, and the origin absorbs the resulting index
/// translation so that the sampled world positions are unchanged.
fn resized_geometry(
    input: &ImageData,
    new_nx: usize,
    new_ny: usize,
    new_nz: usize,
) -> ([i64; 6], [f64; 3], [f64; 3]) {
    let in_extent = input.extent();
    let in_spacing = input.spacing();
    let in_origin = input.origin();
    let out_dims = [new_nx, new_ny, new_nz];
    let mut out_extent = in_extent;
    let mut out_origin = in_origin;
    let mut out_spacing = in_spacing;

    for axis in 0..3 {
        let lo = in_extent[2 * axis];
        let hi = in_extent[2 * axis + 1];
        out_extent[2 * axis] = lo;
        out_extent[2 * axis + 1] = lo + out_dims[axis] as i64 - 1;

        let d = out_dims[axis] as f64 - 1.0;
        let e = (hi - lo) as f64;
        let index_stretch = if d != 0.0 && e != 0.0 { e / d } else { 1.0 };
        let index_translate = lo as f64 - lo as f64 * index_stretch;
        out_spacing[axis] = in_spacing[axis] * index_stretch;
        out_origin[axis] = in_origin[axis] + in_spacing[axis] * index_translate;
    }

    (out_extent, out_origin, out_spacing)
}

fn cubic_weight(t: f64) -> f64 {
    let t = t.abs();
    if t <= 1.0 {
        (1.5 * t - 2.5) * t * t + 1.0
    } else if t <= 2.0 {
        ((-0.5 * t + 2.5) * t - 4.0) * t + 2.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nearest() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let r = resize_nearest(&img, "v", 8, 8);
        assert_eq!(r.dimensions(), [8, 8, 1]);
    }
    #[test]
    fn test_nearest_preserves_corners_and_spacing() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [2.0, 3.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + 10.0 * y,
        );
        let r = resize_nearest(&img, "v", 2, 2);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // `from_function` is evaluated at world coordinates, so the far corner
        // (i, j) = (3, 3) holds 3*2.0 + 10*(3*3.0) = 96, and downsampling to 2x2
        // must land exactly on it.
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 96.0);
        assert_eq!(r.spacing(), [6.0, 9.0, 1.0]);
    }
    #[test]
    fn test_nearest_resizes_each_slice() {
        let img = ImageData::from_function(
            [2, 2, 2],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, z| x + 10.0 * y + 100.0 * z,
        );
        let r = resize_nearest(&img, "v", 3, 3);
        assert_eq!(r.dimensions(), [3, 3, 2]);
        assert_eq!(r.point_data().get_array("v").unwrap().num_tuples(), 18);

        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(17, &mut buf);
        assert_eq!(buf[0], 111.0);
    }
    #[test]
    fn test_nearest_uses_vtk_output_extent_and_origin() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.set_extent([5, 8, 10, 13, 2, 2]);
        img.set_spacing([2.0, 3.0, 1.0]);
        img.set_origin([100.0, 200.0, 300.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 16],
                1,
            )));

        let r = resize_nearest(&img, "v", 2, 2);
        assert_eq!(r.extent(), [5, 6, 10, 11, 2, 2]);
        assert_eq!(r.spacing(), [6.0, 9.0, 1.0]);
        // vtkImageResize keeps the low extent index, so the origin has to absorb
        // the index stretch: origin + spacing * (lo - lo*stretch), e.g.
        // 100 + 2*(5 - 5*3) = 80. Any other origin moves the sampled points.
        assert_eq!(r.origin(), [80.0, 140.0, 300.0]);
        assert_eq!(r.point_from_ijk(0, 0, 0), img.point_from_ijk(0, 0, 0));
        assert_eq!(r.point_from_ijk(1, 1, 0), img.point_from_ijk(3, 3, 0));
    }
    #[test]
    fn test_bicubic() {
        let img = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let r = resize_bicubic(&img, "v", 16, 16);
        assert_eq!(r.dimensions(), [16, 16, 1]);
    }
    #[test]
    fn test_bicubic_resizes_each_slice() {
        let img = ImageData::from_function(
            [2, 2, 2],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, z| x + 10.0 * y + 100.0 * z,
        );
        let r = resize_bicubic(&img, "v", 3, 3);
        assert_eq!(r.dimensions(), [3, 3, 2]);
        assert_eq!(r.point_data().get_array("v").unwrap().num_tuples(), 18);

        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(17, &mut buf);
        assert!((buf[0] - 111.0).abs() < 1e-12);
    }
    #[test]
    fn test_bicubic_uses_vtk_output_extent_and_origin() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.set_extent([5, 8, 10, 13, 2, 2]);
        img.set_spacing([2.0, 3.0, 1.0]);
        img.set_origin([100.0, 200.0, 300.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 16],
                1,
            )));

        let r = resize_bicubic(&img, "v", 2, 2);
        assert_eq!(r.extent(), [5, 6, 10, 11, 2, 2]);
        assert_eq!(r.spacing(), [6.0, 9.0, 1.0]);
        // See `test_nearest_uses_vtk_output_extent_and_origin`: the geometry is
        // computed the same way for both interpolators.
        assert_eq!(r.origin(), [80.0, 140.0, 300.0]);
        assert_eq!(r.point_from_ijk(0, 0, 0), img.point_from_ijk(0, 0, 0));
        assert_eq!(r.point_from_ijk(1, 1, 0), img.point_from_ijk(3, 3, 0));
    }
}
