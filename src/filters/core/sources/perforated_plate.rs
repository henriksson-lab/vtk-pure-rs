//! Perforated plate (grid of holes in a flat plate).
use crate::data::{AnyDataArray, DataArray, ImageData};
/// Generate a perforated plate as a 2D mask image.
pub fn perforated_plate_mask(nx: usize, ny: usize, hole_radius: f64, spacing: f64) -> ImageData {
    let data: Vec<f64> = (0..nx * ny)
        .map(|idx| {
            let ix = idx % nx;
            let iy = idx / nx;
            let x = ix as f64;
            let y = iy as f64;
            let cx = ((x / spacing).round()) * spacing;
            let cy = ((y / spacing).round()) * spacing;
            let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
            if d < hole_radius {
                0.0
            } else {
                1.0
            }
        })
        .collect();
    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing([1.0, 1.0, 1.0])
        .with_origin([0.0, 0.0, 0.0])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Mask", data, 1)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let p = perforated_plate_mask(50, 50, 3.0, 10.0);
        assert_eq!(p.dimensions(), [50, 50, 1]);
        let arr = p.point_data().get_array("Mask").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf); // corner should be hole or solid depending on alignment
    }
}
