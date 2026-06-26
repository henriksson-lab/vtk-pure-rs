//! Gothic (pointed) arch geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn gothic_arch(width: f64, height: f64, thickness: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let hw = width / 2.0;
    let ht = thickness / 2.0;
    let center_offset = if hw.abs() > 1e-12 {
        (height * height - hw * hw) / (2.0 * hw)
    } else {
        0.0
    };
    let radius = (center_offset + hw).abs().max(1e-12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Two arcs meeting at top
    let left_center = center_offset;
    let right_center = -center_offset;
    let left_start = (-hw - left_center).atan2(0.0);
    let left_end = (0.0 - left_center).atan2(height);
    let right_start = (0.0 - right_center).atan2(height);
    let right_end = (hw - right_center).atan2(0.0);
    let left_segments = res / 2;
    let right_segments = res - left_segments;
    for i in 0..=left_segments {
        let t = i as f64 / left_segments as f64;
        let a = left_start + t * (left_end - left_start);
        let x = left_center + radius * a.sin();
        let y = radius * a.cos();
        pts.push([x, y, -ht]);
        pts.push([x, y, ht]);
    }
    for i in 1..=right_segments {
        let t = i as f64 / right_segments as f64;
        let a = right_start + t * (right_end - right_start);
        let x = right_center + radius * a.sin();
        let y = radius * a.cos();
        pts.push([x, y, -ht]);
        pts.push([x, y, ht]);
    }
    // Connect front and back with quads
    let np = pts.len() / 2;
    for i in 0..np - 1 {
        polys.push_cell(&[
            (i * 2) as i64,
            ((i + 1) * 2) as i64,
            ((i + 1) * 2 + 1) as i64,
            (i * 2 + 1) as i64,
        ]);
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
        let a = gothic_arch(2.0, 3.0, 0.3, 12);
        assert!(a.points.len() > 10);
        assert!(a.polys.num_cells() > 3);
        let apex = a.points.get(a.points.len() / 2);
        assert!(apex[0].abs() < 1e-10);
        assert!((apex[1] - 3.0).abs() < 1e-10);
    }
}
