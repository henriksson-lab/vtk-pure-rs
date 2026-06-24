//! Suspension cable/catenary between two points.
use crate::data::{CellArray, Points, PolyData};
pub fn catenary_cable(p0: [f64; 3], p1: [f64; 3], sag: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let mut pts = Points::<f64>::new();
    let mut ids = Vec::new();
    for i in 0..=res {
        let t = i as f64 / res as f64;
        let sag_factor = 4.0 * t * (1.0 - t);
        let x = p0[0] + (p1[0] - p0[0]) * t;
        let y = p0[1] + (p1[1] - p0[1]) * t;
        let z = p0[2] + (p1[2] - p0[2]) * t - sag * sag_factor;
        let idx = pts.len();
        pts.push([x, y, z]);
        ids.push(idx as i64);
    }
    let mut lines = CellArray::new();
    lines.push_cell(&ids);
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
pub fn multi_span_cable(points: &[[f64; 3]], sag: f64, resolution: usize) -> PolyData {
    let mut all_pts = Points::<f64>::new();
    let mut all_lines = CellArray::new();
    for i in 0..points.len() - 1 {
        let cable = catenary_cable(points[i], points[i + 1], sag, resolution);
        let base = all_pts.len() as i64;
        for j in 0..cable.points.len() {
            all_pts.push(cable.points.get(j));
        }
        for cell in cable.lines.iter() {
            let shifted: Vec<i64> = cell.iter().map(|&id| id + base).collect();
            all_lines.push_cell(&shifted);
        }
    }
    let mut r = PolyData::new();
    r.points = all_pts;
    r.lines = all_lines;
    r
}
pub fn power_line(
    poles: &[[f64; 3]],
    num_wires: usize,
    wire_spacing: f64,
    sag: f64,
    resolution: usize,
) -> PolyData {
    let nw = num_wires.max(1);
    let hw = (nw - 1) as f64 * wire_spacing / 2.0;
    let mut all_pts = Points::<f64>::new();
    let mut all_lines = CellArray::new();
    for wi in 0..nw {
        let offset = (wi as f64 - hw / wire_spacing.max(1e-15)) * wire_spacing;
        for i in 0..poles.len() - 1 {
            let p0 = [poles[i][0], poles[i][1] + offset, poles[i][2]];
            let p1 = [poles[i + 1][0], poles[i + 1][1] + offset, poles[i + 1][2]];
            let cable = catenary_cable(p0, p1, sag, resolution);
            let base = all_pts.len() as i64;
            for j in 0..cable.points.len() {
                all_pts.push(cable.points.get(j));
            }
            for cell in cable.lines.iter() {
                let shifted: Vec<i64> = cell.iter().map(|&id| id + base).collect();
                all_lines.push_cell(&shifted);
            }
        }
    }
    // Poles
    for p in poles {
        let pb = all_pts.len();
        all_pts.push([p[0], p[1], 0.0]);
        all_pts.push([p[0], p[1], p[2]]);
        all_lines.push_cell(&[pb as i64, (pb + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = all_pts;
    r.lines = all_lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_single() {
        let c = catenary_cable([0.0, 0.0, 10.0], [20.0, 0.0, 10.0], 3.0, 12);
        assert!(c.points.len() > 10);
        assert_eq!(c.lines.num_cells(), 1);
    }
    #[test]
    fn test_multi() {
        let c = multi_span_cable(
            &[[0.0, 0.0, 10.0], [10.0, 0.0, 10.0], [20.0, 0.0, 10.0]],
            2.0,
            8,
        );
        assert_eq!(c.lines.num_cells(), 2);
    }
    #[test]
    fn test_power() {
        let p = power_line(
            &[[0.0, 0.0, 8.0], [30.0, 0.0, 8.0], [60.0, 0.0, 8.0]],
            3,
            0.5,
            2.0,
            10,
        );
        assert!(p.lines.num_cells() > 5);
    }
}
