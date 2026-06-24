//! Segmented pipe elbow (mitered bend).
use crate::data::{CellArray, Points, PolyData};
pub fn mitered_elbow(
    radius: f64,
    pipe_radius: f64,
    angle_degrees: f64,
    segments: usize,
    tube_res: usize,
) -> PolyData {
    let segs = segments.max(1);
    let tres = tube_res.max(3);
    let angle = angle_degrees.to_radians();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for is in 0..=segs {
        let t = is as f64 / segs as f64;
        let a = angle * t;
        let cx = radius * a.sin();
        let cz = radius * (1.0 - a.cos());
        let tx = a.cos();
        let tz = a.sin();
        let nx = -tz;
        let nz = tx;
        for it in 0..tres {
            let phi = 2.0 * std::f64::consts::PI * it as f64 / tres as f64;
            pts.push([
                cx + pipe_radius * phi.cos() * nx,
                pipe_radius * phi.sin(),
                cz + pipe_radius * phi.cos() * nz,
            ]);
        }
    }
    for is in 0..segs {
        let r0 = is * tres;
        let r1 = (is + 1) * tres;
        for it in 0..tres {
            let it1 = (it + 1) % tres;
            polys.push_cell(&[
                (r0 + it) as i64,
                (r0 + it1) as i64,
                (r1 + it1) as i64,
                (r1 + it) as i64,
            ]);
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
        let e = mitered_elbow(2.0, 0.3, 90.0, 4, 8);
        assert!(e.points.len() > 20);
        assert!(e.polys.num_cells() > 10);
    }
}
