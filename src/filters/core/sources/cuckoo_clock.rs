//! Cuckoo clock (A-frame house shape with pendulum and weights).
use crate::data::{CellArray, Points, PolyData};

pub fn cuckoo_clock(width: f64, height: f64) -> PolyData {
    let hw = width / 2.0;
    let hd = width * 0.2;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // House body
    let bb = pts.len();
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    pts.push([-hw, -hd, height * 0.7]);
    pts.push([hw, -hd, height * 0.7]);
    pts.push([hw, hd, height * 0.7]);
    pts.push([-hw, hd, height * 0.7]);
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 5) as i64, (bb + 4) as i64]);
    polys.push_cell(&[
        (bb + 1) as i64,
        (bb + 2) as i64,
        (bb + 6) as i64,
        (bb + 5) as i64,
    ]);
    polys.push_cell(&[
        (bb + 2) as i64,
        (bb + 3) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
    ]);
    polys.push_cell(&[(bb + 3) as i64, bb as i64, (bb + 4) as i64, (bb + 7) as i64]);
    // Peaked roof
    let ridge_f = pts.len();
    pts.push([0.0, -hd, height]);
    let ridge_b = pts.len();
    pts.push([0.0, hd, height]);
    polys.push_cell(&[(bb + 4) as i64, (bb + 5) as i64, ridge_f as i64]);
    polys.push_cell(&[(bb + 6) as i64, (bb + 7) as i64, ridge_b as i64]);
    polys.push_cell(&[
        (bb + 4) as i64,
        ridge_f as i64,
        ridge_b as i64,
        (bb + 7) as i64,
    ]); // left roof
    polys.push_cell(&[
        (bb + 5) as i64,
        (bb + 6) as i64,
        ridge_b as i64,
        ridge_f as i64,
    ]); // right roof
        // Clock face (circle)
    let face_z = height * 0.45;
    let face_r = width * 0.2;
    let na = 12;
    let fb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([face_r * a.cos(), -hd - 0.01, face_z + face_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(fb + j) as i64, (fb + (j + 1) % na) as i64]);
    }
    // Cuckoo door (small rectangle above face)
    let door_w = width * 0.08;
    let door_h = width * 0.1;
    let door_z = face_z + face_r + door_h * 0.5;
    let db = pts.len();
    pts.push([-door_w, -hd - 0.01, door_z]);
    pts.push([door_w, -hd - 0.01, door_z]);
    pts.push([door_w, -hd - 0.01, door_z + door_h]);
    pts.push([-door_w, -hd - 0.01, door_z + door_h]);
    lines.push_cell(&[db as i64, (db + 1) as i64]);
    lines.push_cell(&[(db + 1) as i64, (db + 2) as i64]);
    lines.push_cell(&[(db + 2) as i64, (db + 3) as i64]);
    lines.push_cell(&[(db + 3) as i64, db as i64]);
    // Pendulum
    let pv = pts.len();
    pts.push([0.0, -hd - 0.01, 0.0]);
    let pb = pts.len();
    pts.push([width * 0.1, -hd - 0.01, -height * 0.3]);
    lines.push_cell(&[pv as i64, pb as i64]);
    // Weights (two hanging chains)
    for &sx in &[-0.3f64, 0.3] {
        let w0 = pts.len();
        pts.push([sx * width, -hd - 0.01, 0.0]);
        let w1 = pts.len();
        pts.push([sx * width, -hd - 0.01, -height * 0.4]);
        lines.push_cell(&[w0 as i64, w1 as i64]);
    }
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
    fn test_cuckoo() {
        let m = cuckoo_clock(3.0, 4.0);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() >= 8);
        assert!(m.lines.num_cells() > 10);
    }
}
