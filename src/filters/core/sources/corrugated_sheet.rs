//! Corrugated sheet (wavy metal/roofing panel).
use crate::data::{CellArray, Points, PolyData};
pub fn corrugated_sheet(
    width: f64,
    length: f64,
    amplitude: f64,
    wavelength: f64,
    nx: usize,
    ny: usize,
) -> PolyData {
    let nx = nx.max(2);
    let ny = ny.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let dx = width / (nx - 1) as f64;
    let dy = length / (ny - 1) as f64;
    let freq = 2.0 * std::f64::consts::PI / wavelength;
    for iy in 0..ny {
        for ix in 0..nx {
            let x = ix as f64 * dx;
            let y = iy as f64 * dy;
            let z = amplitude * (freq * x).sin();
            pts.push([x, y, z]);
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
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = corrugated_sheet(4.0, 6.0, 0.3, 1.0, 20, 10);
        assert_eq!(c.points.len(), 200);
        assert_eq!(c.polys.num_cells(), 171);
    }
}
