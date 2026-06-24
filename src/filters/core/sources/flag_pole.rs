//! Flag on a pole.
use crate::data::{CellArray, Points, PolyData};
pub fn flag_pole(
    pole_h: f64,
    flag_w: f64,
    flag_h: f64,
    wave_amplitude: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pole
    let pb = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    pts.push([0.0, 0.0, pole_h]);
    lines.push_cell(&[pb as i64, (pb + 1) as i64]);
    // Flag (wavy surface)
    let fz_top = pole_h;
    let fz_bot = pole_h - flag_h;
    let nx = res;
    let ny = res / 2;
    for iy in 0..=ny {
        let t_y = iy as f64 / ny as f64;
        let z = fz_top - (fz_top - fz_bot) * t_y;
        for ix in 0..=nx {
            let t_x = ix as f64 / nx as f64;
            let x = flag_w * t_x;
            let wave = wave_amplitude * (t_x * 2.0) * (t_x * std::f64::consts::PI * 2.0).sin();
            pts.push([x, wave, z]);
        }
    }
    let fb = pb + 2;
    let fw = nx + 1;
    for iy in 0..ny {
        for ix in 0..nx {
            polys.push_cell(&[
                (fb + iy * fw + ix) as i64,
                (fb + iy * fw + ix + 1) as i64,
                (fb + (iy + 1) * fw + ix + 1) as i64,
                (fb + (iy + 1) * fw + ix) as i64,
            ]);
        }
    }
    // Ball on top
    let _bb = pts.len();
    pts.push([0.0, 0.0, pole_h + pole_h * 0.02]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let f = flag_pole(5.0, 2.0, 1.0, 0.1, 8);
        assert!(f.polys.num_cells() >= 8);
        assert_eq!(f.lines.num_cells(), 1);
    }
}
