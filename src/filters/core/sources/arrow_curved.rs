//! Curved arrow (arc with arrowhead).
use crate::data::{CellArray, Points, PolyData};
pub fn curved_arrow(
    radius: f64,
    angle_degrees: f64,
    width: f64,
    head_size: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(4);
    let angle = angle_degrees.to_radians();
    let hw = width.max(0.0) / 2.0;
    let hs = head_size.max(0.0);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let head_angle = hs.to_radians().min(angle.abs() * 0.3);
    let arc_end = angle - angle.signum() * head_angle;
    // Arc ribbon
    for i in 0..=res {
        let t = arc_end * i as f64 / res as f64;
        let c = t.cos();
        let s = t.sin();
        pts.push([(radius - hw) * c, (radius - hw) * s, 0.0]);
        pts.push([(radius + hw) * c, (radius + hw) * s, 0.0]);
    }
    for i in 0..res {
        let b = i * 2;
        polys.push_cell(&[b as i64, (b + 1) as i64, (b + 3) as i64, (b + 2) as i64]);
    }
    // Arrowhead triangle
    let tip_angle = angle;
    let base_angle = arc_end;
    let tc = tip_angle.cos();
    let ts = tip_angle.sin();
    let bc = base_angle.cos();
    let bs = base_angle.sin();
    let a0 = pts.len();
    pts.push([(radius - hs) * bc, (radius - hs) * bs, 0.0]);
    let a1 = pts.len();
    pts.push([(radius + hs) * bc, (radius + hs) * bs, 0.0]);
    let a2 = pts.len();
    pts.push([radius * tc, radius * ts, 0.0]);
    polys.push_cell(&[a0 as i64, a1 as i64, a2 as i64]);
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
        let a = curved_arrow(2.0, 90.0, 0.3, 0.5, 12);
        assert!(a.points.len() > 10);
        assert!(a.polys.num_cells() > 5);
    }

    #[test]
    fn negative_angle() {
        let a = curved_arrow(2.0, -90.0, 0.3, 0.5, 12);
        assert!(a.points.len() > 10);
        assert!(a.polys.num_cells() > 5);
    }
}
