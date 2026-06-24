//! Simple texture synthesis by patch copying.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Synthesize a larger texture from a small sample using random patch placement.
pub fn synthesize_texture(
    sample: &ImageData,
    scalars: &str,
    out_w: usize,
    out_h: usize,
    patch_size: usize,
    seed: u64,
) -> ImageData {
    let arr = match sample.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return sample.clone(),
    };
    let dims = sample.dimensions();
    let (sx, sy) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let src: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let ps = patch_size.max(1);

    let mut out = vec![0.0f64; out_w * out_h];
    let mut rng = SimpleRng(seed);

    let mut y = 0;
    while y < out_h {
        let mut x = 0;
        while x < out_w {
            let ox = (rng.next() * (sx.saturating_sub(ps)) as f64) as usize;
            let oy = (rng.next() * (sy.saturating_sub(ps)) as f64) as usize;
            for py in 0..ps {
                for px in 0..ps {
                    let dx = x + px;
                    let dy = y + py;
                    if dx < out_w && dy < out_h && ox + px < sx && oy + py < sy {
                        out[dx + dy * out_w] = src[(ox + px) + (oy + py) * sx];
                    }
                }
            }
            x += ps;
        }
        y += ps;
    }

    ImageData::with_dimensions(out_w, out_h, 1)
        .with_spacing(sample.spacing())
        .with_origin(sample.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, out, 1)))
}

/// Generate a checkerboard pattern image.
pub fn checkerboard(nx: usize, ny: usize, check_size: usize, val_a: f64, val_b: f64) -> ImageData {
    let cs = check_size.max(1);
    let data: Vec<f64> = (0..nx * ny)
        .map(|idx| {
            let ix = idx % nx;
            let iy = idx / nx;
            if (ix / cs + iy / cs) % 2 == 0 {
                val_a
            } else {
                val_b
            }
        })
        .collect();
    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing([1.0, 1.0, 1.0])
        .with_origin([0.0, 0.0, 0.0])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Pattern", data, 1)))
}

struct SimpleRng(u64);
impl SimpleRng {
    fn next(&mut self) -> f64 {
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
    fn test_synthesize() {
        let sample = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        let r = synthesize_texture(&sample, "v", 16, 16, 4, 42);
        assert_eq!(r.dimensions(), [16, 16, 1]);
    }
    #[test]
    fn test_checkerboard() {
        let r = checkerboard(8, 8, 2, 0.0, 255.0);
        assert_eq!(r.dimensions(), [8, 8, 1]);
        let arr = r.point_data().get_array("Pattern").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 255.0);
    }
}
