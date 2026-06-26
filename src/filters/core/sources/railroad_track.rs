//! Railroad track geometry (rails + ties).
use crate::data::{CellArray, Points, PolyData};
pub fn railroad_track(
    length: f64,
    gauge: f64,
    tie_spacing: f64,
    tie_width: f64,
    tie_height: f64,
) -> PolyData {
    let hg = gauge / 2.0;
    let hw = tie_width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Rails (lines)
    let lb = pts.len();
    pts.push([0.0, -hg, tie_height]);
    pts.push([length, -hg, tie_height]);
    lines.push_cell(&[lb as i64, (lb + 1) as i64]);
    let rb = pts.len();
    pts.push([0.0, hg, tie_height]);
    pts.push([length, hg, tie_height]);
    lines.push_cell(&[rb as i64, (rb + 1) as i64]);
    // Ties
    let nt = (length / tie_spacing).ceil() as usize;
    for i in 0..=nt {
        let x = i as f64 * tie_spacing;
        if x > length {
            break;
        }
        let tb = pts.len();
        pts.push([x - hw, -hg * 1.3, 0.0]);
        pts.push([x + hw, -hg * 1.3, 0.0]);
        pts.push([x + hw, hg * 1.3, 0.0]);
        pts.push([x - hw, hg * 1.3, 0.0]);
        pts.push([x - hw, -hg * 1.3, tie_height]);
        pts.push([x + hw, -hg * 1.3, tie_height]);
        pts.push([x + hw, hg * 1.3, tie_height]);
        pts.push([x - hw, hg * 1.3, tie_height]);
        let f = |j: usize| (tb + j) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]); // bottom
        polys.push_cell(&[f(4), f(5), f(6), f(7)]); // top
        polys.push_cell(&[f(0), f(1), f(5), f(4)]); // front
        polys.push_cell(&[f(1), f(2), f(6), f(5)]); // right end
        polys.push_cell(&[f(2), f(3), f(7), f(6)]); // back
        polys.push_cell(&[f(3), f(0), f(4), f(7)]); // left end
    }
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
        let t = railroad_track(10.0, 1.435, 0.6, 0.2, 0.15);
        assert!(t.polys.num_cells() > 10);
        assert_eq!(t.lines.num_cells(), 2);
    }
}
