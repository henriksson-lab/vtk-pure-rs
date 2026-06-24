//! Birdhouse with box body, pitched roof, and entrance hole.
use crate::data::{CellArray, Points, PolyData};

pub fn birdhouse(width: f64, height: f64, depth: f64) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let roof_h = height * 0.3;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Box body
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    pts.push([-hw, -hd, height]);
    pts.push([hw, -hd, height]);
    pts.push([hw, hd, height]);
    pts.push([-hw, hd, height]);
    // Walls
    polys.push_cell(&[0, 1, 5, 4]);
    polys.push_cell(&[1, 2, 6, 5]);
    polys.push_cell(&[2, 3, 7, 6]);
    polys.push_cell(&[3, 0, 4, 7]);
    polys.push_cell(&[0, 3, 2, 1]); // floor
                                    // Roof ridge
    let r0 = pts.len();
    pts.push([0.0, -hd - 0.1, height + roof_h]);
    let r1 = pts.len();
    pts.push([0.0, hd + 0.1, height + roof_h]);
    // Gable ends
    polys.push_cell(&[4, 5, r0 as i64]);
    polys.push_cell(&[7, 6, r1 as i64]);
    // Roof panels
    polys.push_cell(&[4, r0 as i64, r1 as i64, 7]);
    polys.push_cell(&[5, 6, r1 as i64, r0 as i64]);
    // Entrance hole (circle of lines)
    let mut lines = CellArray::new();
    let hole_r = width * 0.15;
    let hole_z = height * 0.65;
    let na = 12;
    let hb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([hole_r * a.cos(), -hd - 0.01, hole_z + hole_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(hb + j) as i64, (hb + (j + 1) % na) as i64]);
    }
    // Perch
    let p0 = pts.len();
    pts.push([0.0, -hd - 0.01, hole_z - hole_r * 1.5]);
    let p1 = pts.len();
    pts.push([0.0, -hd - width * 0.3, hole_z - hole_r * 1.5]);
    lines.push_cell(&[p0 as i64, p1 as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_birdhouse() {
        let m = birdhouse(2.0, 3.0, 2.0);
        assert!(m.points.len() > 15);
        assert!(m.polys.num_cells() > 5);
    }
}
