use crate::data::{AnyDataArray, DataArray, ImageData};

/// Parameters for generating a 3D noise field on ImageData.
pub struct NoiseFieldParams {
    /// Dimensions. Default: [32, 32, 32]
    pub dimensions: [usize; 3],
    /// Frequency (scale of noise features). Default: 4.0
    pub frequency: f64,
    /// Seed for reproducibility. Default: 42
    pub seed: u64,
}

impl Default for NoiseFieldParams {
    fn default() -> Self {
        Self {
            dimensions: [32, 32, 32],
            frequency: 4.0,
            seed: 42,
        }
    }
}

/// Generate a 3D value-noise field on ImageData.
///
/// Uses a simple hash-based interpolated noise (similar to Perlin noise
/// but using value noise for simplicity). The output "Noise" scalar
/// ranges approximately [-1, 1].
pub fn noise_field(params: &NoiseFieldParams) -> ImageData {
    let nx = params.dimensions[0].max(2);
    let ny = params.dimensions[1].max(2);
    let nz = params.dimensions[2].max(2);
    let f = params.frequency;
    let seed = params.seed;

    let mut img = ImageData::with_dimensions(nx, ny, nz);
    let sp = 1.0 / (nx as f64 - 1.0).max(1.0);
    img.set_spacing([sp, sp, sp]);

    let n = nx * ny * nz;
    let mut values = Vec::with_capacity(n);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let x = i as f64 / nx as f64 * f;
                let y = j as f64 / ny as f64 * f;
                let z = k as f64 / nz as f64 * f;
                values.push(value_noise_3d(x, y, z, seed));
            }
        }
    }

    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Noise", values, 1)));
    img.point_data_mut().set_active_scalars("Noise");
    img
}

fn value_noise_3d(x: f64, y: f64, z: f64, seed: u64) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let iz = z.floor() as i64;
    let fx = x - ix as f64;
    let fy = y - iy as f64;
    let fz = z - iz as f64;

    // Smoothstep
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let sz = fz * fz * (3.0 - 2.0 * fz);

    let mut result = 0.0;
    for dz in 0..2i64 {
        for dy in 0..2i64 {
            for dx in 0..2i64 {
                let h = hash3(ix + dx, iy + dy, iz + dz, seed);
                let wx = if dx == 0 { 1.0 - sx } else { sx };
                let wy = if dy == 0 { 1.0 - sy } else { sy };
                let wz = if dz == 0 { 1.0 - sz } else { sz };
                result += h * wx * wy * wz;
            }
        }
    }
    result
}

fn hash3(x: i64, y: i64, z: i64, seed: u64) -> f64 {
    let mut h = seed
        .wrapping_add(x as u64)
        .wrapping_mul(6364136223846793005);
    h = h.wrapping_add(y as u64).wrapping_mul(6364136223846793005);
    h = h.wrapping_add(z as u64).wrapping_mul(6364136223846793005);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    (h as f64 / u64::MAX as f64) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_noise() {
        let img = noise_field(&NoiseFieldParams::default());
        assert_eq!(img.dimensions(), [32, 32, 32]);
        assert!(img.point_data().get_array("Noise").is_some());
    }

    #[test]
    fn has_variation() {
        let img = noise_field(&NoiseFieldParams {
            dimensions: [8, 8, 8],
            ..Default::default()
        });
        let arr = img.point_data().get_array("Noise").unwrap();
        let mut buf = [0.0f64];
        let mut min_v = f64::MAX;
        let mut max_v = f64::MIN;
        for i in 0..512 {
            arr.tuple_as_f64(i, &mut buf);
            min_v = min_v.min(buf[0]);
            max_v = max_v.max(buf[0]);
        }
        assert!(max_v > min_v);
    }

    #[test]
    fn reproducible() {
        let a = noise_field(&NoiseFieldParams {
            dimensions: [4, 4, 4],
            seed: 123,
            ..Default::default()
        });
        let b = noise_field(&NoiseFieldParams {
            dimensions: [4, 4, 4],
            seed: 123,
            ..Default::default()
        });
        let aa = a.point_data().get_array("Noise").unwrap();
        let ba = b.point_data().get_array("Noise").unwrap();
        let mut ba1 = [0.0f64];
        let mut bb1 = [0.0f64];
        for i in 0..64 {
            aa.tuple_as_f64(i, &mut ba1);
            ba.tuple_as_f64(i, &mut bb1);
            assert_eq!(ba1[0], bb1[0]);
        }
    }
}
