//! Image noise generation and addition.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Add uniform random noise to an image.
pub fn add_uniform_noise(input: &ImageData, scalars: &str, amplitude: f64, seed: u64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut rng = SimpleRng(seed);
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] + amplitude * (rng.next_f64() * 2.0 - 1.0)
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Add salt-and-pepper noise to an image.
pub fn salt_and_pepper(input: &ImageData, scalars: &str, density: f64, seed: u64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut rng = SimpleRng(seed);
    let data: Vec<f64> = vals
        .iter()
        .map(|&v| {
            let r = rng.next_f64();
            if r < density / 2.0 {
                mn
            } else if r < density {
                mx
            } else {
                v
            }
        })
        .collect();
    let dims = input.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Generate a pure noise image.
pub fn noise_image(nx: usize, ny: usize, seed: u64) -> ImageData {
    let mut rng = SimpleRng(seed);
    let data: Vec<f64> = (0..nx * ny).map(|_| rng.next_f64()).collect();
    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing([1.0, 1.0, 1.0])
        .with_origin([0.0, 0.0, 0.0])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Noise", data, 1)))
}

struct SimpleRng(u64);
impl SimpleRng {
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f64) / (u32::MAX as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_uniform_noise() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 50.0,
        );
        let noisy = add_uniform_noise(&img, "v", 10.0, 42);
        let arr = noisy.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 50.0).abs() <= 10.0);
    }
    #[test]
    fn test_salt_pepper() {
        let img = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 128.0,
        );
        let noisy = salt_and_pepper(&img, "v", 0.1, 42);
        assert_eq!(noisy.dimensions(), [20, 20, 1]);
    }
    #[test]
    fn test_noise_image() {
        let img = noise_image(16, 16, 99);
        assert_eq!(img.dimensions(), [16, 16, 1]);
        let arr = img.point_data().get_array("Noise").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] >= 0.0 && buf[0] <= 1.0);
    }
}
