//! Funnel/cone-with-hole geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn funnel(top_radius: f64, bottom_radius: f64, height: f64, resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([top_radius * a.cos(), top_radius * a.sin(), height]);
        pts.push([bottom_radius * a.cos(), bottom_radius * a.sin(), 0.0]);
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (i * 2) as i64,
            ((i * 2) + 1) as i64,
            ((j * 2) + 1) as i64,
            (j * 2) as i64,
        ]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn hopper(top_w: f64, top_d: f64, bottom_w: f64, bottom_d: f64, height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let tw = top_w / 2.0;
    let td = top_d / 2.0;
    let bw = bottom_w / 2.0;
    let bd = bottom_d / 2.0;
    pts.push([-tw, -td, height]);
    pts.push([tw, -td, height]);
    pts.push([tw, td, height]);
    pts.push([-tw, td, height]);
    pts.push([-bw, -bd, 0.0]);
    pts.push([bw, -bd, 0.0]);
    pts.push([bw, bd, 0.0]);
    pts.push([-bw, bd, 0.0]);
    polys.push_cell(&[0, 1, 5, 4]);
    polys.push_cell(&[1, 2, 6, 5]);
    polys.push_cell(&[2, 3, 7, 6]);
    polys.push_cell(&[3, 0, 4, 7]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_funnel() {
        let f = funnel(2.0, 0.5, 3.0, 12);
        assert_eq!(f.points.len(), 24);
        assert_eq!(f.polys.num_cells(), 12);
    }
    #[test]
    fn test_hopper() {
        let h = hopper(4.0, 3.0, 1.0, 0.5, 2.0);
        assert_eq!(h.points.len(), 8);
        assert_eq!(h.polys.num_cells(), 4);
    }
}
