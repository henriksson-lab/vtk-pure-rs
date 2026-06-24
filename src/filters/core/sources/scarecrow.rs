//! Scarecrow with cross frame and hat.
use crate::data::{CellArray, Points, PolyData};

pub fn scarecrow(height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Vertical pole
    let p0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let p1 = pts.len();
    pts.push([0.0, 0.0, height]);
    lines.push_cell(&[p0 as i64, p1 as i64]);
    // Crossbar (arms)
    let arm_span = height * 0.5;
    let arm_z = height * 0.75;
    let a0 = pts.len();
    pts.push([-arm_span / 2.0, 0.0, arm_z]);
    let a1 = pts.len();
    pts.push([arm_span / 2.0, 0.0, arm_z]);
    lines.push_cell(&[a0 as i64, a1 as i64]);
    // Head (circle)
    let head_r = height * 0.08;
    let head_z = height + head_r;
    let na = 12;
    let hb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([head_r * a.cos(), 0.01, head_z + head_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(hb + j) as i64, (hb + (j + 1) % na) as i64]);
    }
    // Hat (triangle on top of head)
    let hat_b = pts.len();
    pts.push([-head_r * 1.5, 0.01, head_z + head_r]);
    pts.push([head_r * 1.5, 0.01, head_z + head_r]);
    pts.push([0.0, 0.01, head_z + head_r * 2.5]);
    polys.push_cell(&[hat_b as i64, (hat_b + 1) as i64, (hat_b + 2) as i64]);
    // Shirt body outline
    let sh0 = pts.len();
    pts.push([-height * 0.12, 0.0, arm_z]);
    let sh1 = pts.len();
    pts.push([height * 0.12, 0.0, arm_z]);
    let sh2 = pts.len();
    pts.push([height * 0.1, 0.0, height * 0.4]);
    let sh3 = pts.len();
    pts.push([-height * 0.1, 0.0, height * 0.4]);
    lines.push_cell(&[sh0 as i64, sh1 as i64]);
    lines.push_cell(&[sh1 as i64, sh2 as i64]);
    lines.push_cell(&[sh2 as i64, sh3 as i64]);
    lines.push_cell(&[sh3 as i64, sh0 as i64]);
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
    fn test_scarecrow() {
        let m = scarecrow(5.0);
        assert!(m.points.len() > 15);
        assert!(m.lines.num_cells() > 10);
    }
}
