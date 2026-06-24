//! Helix as a polyline (wireframe).
use crate::data::{CellArray, Points, PolyData};
pub fn helix_wire(radius: f64, pitch: f64, turns: f64, resolution: usize) -> PolyData {
    let total = (resolution as f64 * turns).ceil() as usize;
    let total = total.max(4);
    let mut pts = Points::<f64>::new();
    let mut ids = Vec::new();
    for i in 0..=total {
        let t = i as f64 / total as f64 * turns;
        let a = 2.0 * std::f64::consts::PI * t;
        pts.push([radius * a.cos(), radius * a.sin(), t * pitch]);
        ids.push(i as i64);
    }
    let mut lines = CellArray::new();
    lines.push_cell(&ids);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn double_helix(
    radius: f64,
    pitch: f64,
    turns: f64,
    offset_angle: f64,
    resolution: usize,
) -> PolyData {
    let total = (resolution as f64 * turns).ceil() as usize;
    let total = total.max(4);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for strand in 0..2 {
        let phase = strand as f64 * offset_angle;
        let mut ids = Vec::new();
        for i in 0..=total {
            let t = i as f64 / total as f64 * turns;
            let a = 2.0 * std::f64::consts::PI * t + phase;
            let idx = pts.len();
            pts.push([radius * a.cos(), radius * a.sin(), t * pitch]);
            ids.push(idx as i64);
        }
        lines.push_cell(&ids);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_single() {
        let h = helix_wire(1.0, 0.5, 3.0, 20);
        assert!(h.points.len() > 30);
        assert_eq!(h.lines.num_cells(), 1);
    }
    #[test]
    fn test_double() {
        let h = double_helix(1.0, 0.5, 2.0, std::f64::consts::PI, 16);
        assert_eq!(h.lines.num_cells(), 2);
    }
}
