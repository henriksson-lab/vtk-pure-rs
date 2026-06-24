//! Procedural terrain generation sources.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Generate a terrain mesh using multi-octave value noise (fBm).
pub fn terrain(
    width: f64,
    depth: f64,
    resolution: usize,
    amplitude: f64,
    octaves: usize,
    seed: u64,
) -> PolyData {
    let n = resolution.max(2);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut height_data = Vec::new();

    for j in 0..=n {
        for i in 0..=n {
            let x = width * i as f64 / n as f64 - width / 2.0;
            let y = depth * j as f64 / n as f64 - depth / 2.0;
            let z = fbm(x * 0.1, y * 0.1, seed, octaves) * amplitude;
            points.push([x, y, z]);
            height_data.push(z);
        }
    }

    let row = n + 1;
    for j in 0..n {
        for i in 0..n {
            let p0 = (j * row + i) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Height",
            height_data,
            1,
        )));
    mesh.point_data_mut().set_active_scalars("Height");
    mesh
}

/// Generate a terrain with erosion-like features (ridges and valleys).
pub fn terrain_ridged(
    width: f64,
    depth: f64,
    resolution: usize,
    amplitude: f64,
    seed: u64,
) -> PolyData {
    let n = resolution.max(2);
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut height_data = Vec::new();

    for j in 0..=n {
        for i in 0..=n {
            let x = width * i as f64 / n as f64 - width / 2.0;
            let y = depth * j as f64 / n as f64 - depth / 2.0;
            // Ridged multifractal
            let mut z = 0.0;
            let mut freq = 0.1;
            let mut amp = amplitude;
            for oct in 0..6 {
                let v = (1.0 - fbm(x * freq, y * freq, seed + oct as u64, 1).abs()) * amp;
                z += v * v; // squared for sharper ridges
                freq *= 2.0;
                amp *= 0.5;
            }
            points.push([x, y, z]);
            height_data.push(z);
        }
    }

    let row = n + 1;
    for j in 0..n {
        for i in 0..n {
            let p0 = (j * row + i) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Height",
            height_data,
            1,
        )));
    mesh
}

fn fbm(x: f64, y: f64, seed: u64, octaves: usize) -> f64 {
    let mut val = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    for i in 0..octaves {
        val += amp * value_noise(x * freq, y * freq, seed + i as u64);
        amp *= 0.5;
        freq *= 2.0;
    }
    val
}

fn value_noise(x: f64, y: f64, seed: u64) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let fx = x - x.floor();
    let fy = y - y.floor();
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let hash = |x: i64, y: i64| -> f64 {
        let h = ((x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263)) as u64).wrapping_add(seed);
        let h = h.wrapping_mul(6364136223846793005).wrapping_add(1);
        (h >> 33) as f64 / (1u64 << 31) as f64 - 1.0
    };

    let v00 = hash(ix, iy);
    let v10 = hash(ix + 1, iy);
    let v01 = hash(ix, iy + 1);
    let v11 = hash(ix + 1, iy + 1);

    let a = v00 + sx * (v10 - v00);
    let b = v01 + sx * (v11 - v01);
    a + sy * (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_terrain() {
        let t = terrain(10.0, 10.0, 16, 2.0, 4, 42);
        assert_eq!(t.points.len(), 17 * 17);
        assert!(t.point_data().get_array("Height").is_some());
        // Should have variation
        let arr = t.point_data().get_array("Height").unwrap();
        let mut min_h = f64::MAX;
        let mut max_h = f64::MIN;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            min_h = min_h.min(buf[0]);
            max_h = max_h.max(buf[0]);
        }
        assert!(max_h - min_h > 0.1);
    }

    #[test]
    fn ridged_terrain() {
        let t = terrain_ridged(10.0, 10.0, 16, 2.0, 42);
        assert!(t.points.len() > 100);
    }

    #[test]
    fn different_seeds() {
        let t1 = terrain(5.0, 5.0, 8, 1.0, 3, 1);
        let t2 = terrain(5.0, 5.0, 8, 1.0, 3, 999);
        let h1 = t1.points.get(40)[2];
        let h2 = t2.points.get(40)[2];
        assert!((h1 - h2).abs() > 0.001);
    }
}
