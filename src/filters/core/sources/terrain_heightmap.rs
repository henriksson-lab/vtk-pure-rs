//! Procedural terrain from noise-based heightmap.
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn terrain(
    nx: usize,
    ny: usize,
    dx: f64,
    dy: f64,
    amplitude: f64,
    frequency: f64,
    seed: u64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut heights = Vec::with_capacity(nx * ny);
    let mut rng = seed;
    // Simple value noise
    let noise_grid_size = 16;
    let ng: Vec<f64> = (0..noise_grid_size * noise_grid_size)
        .map(|_| {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((rng >> 33) as f64 / u32::MAX as f64) * 2.0 - 1.0
        })
        .collect();
    for iy in 0..ny {
        for ix in 0..nx {
            let x = ix as f64 * dx;
            let y = iy as f64 * dy;
            let fx = (x * frequency).rem_euclid(noise_grid_size as f64);
            let fy = (y * frequency).rem_euclid(noise_grid_size as f64);
            let gx = fx as usize % noise_grid_size;
            let gy = fy as usize % noise_grid_size;
            let gx1 = (gx + 1) % noise_grid_size;
            let gy1 = (gy + 1) % noise_grid_size;
            let tx = fx - fx.floor();
            let ty = fy - fy.floor();
            let v00 = ng[gx + gy * noise_grid_size];
            let v10 = ng[gx1 + gy * noise_grid_size];
            let v01 = ng[gx + gy1 * noise_grid_size];
            let v11 = ng[gx1 + gy1 * noise_grid_size];
            let h = amplitude
                * ((v00 * (1.0 - tx) + v10 * tx) * (1.0 - ty) + (v01 * (1.0 - tx) + v11 * tx) * ty);
            pts.push([x, y, h]);
            heights.push(h);
        }
    }
    for iy in 0..ny - 1 {
        for ix in 0..nx - 1 {
            let i00 = (iy * nx + ix) as i64;
            let i10 = (iy * nx + ix + 1) as i64;
            let i01 = ((iy + 1) * nx + ix) as i64;
            let i11 = ((iy + 1) * nx + ix + 1) as i64;
            polys.push_cell(&[i00, i10, i11, i01]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Elevation",
            heights,
            1,
        )));
    r.point_data_mut().set_active_scalars("Elevation");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let t = terrain(20, 20, 1.0, 1.0, 5.0, 0.2, 42);
        assert_eq!(t.points.len(), 400);
        assert_eq!(t.polys.num_cells(), 361);
        assert!(t.point_data().get_array("Elevation").is_some());
    }
}
