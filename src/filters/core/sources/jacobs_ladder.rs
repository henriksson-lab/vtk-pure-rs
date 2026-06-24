//! Jacob's ladder toy (chain of hinged blocks).
use crate::data::{CellArray, Points, PolyData};

pub fn jacobs_ladder(block_width: f64, block_height: f64, n_blocks: usize) -> PolyData {
    let nb = n_blocks.max(3);
    let hw = block_width / 2.0;
    let hh = block_height / 2.0;
    let spacing = block_height * 1.2;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    for b in 0..nb {
        let z = -spacing * b as f64;
        let _angle = if b % 2 == 0 { 0.0 } else { 0.1 }; // slight alternating tilt
        let bb = pts.len();
        pts.push([-hw, -0.01, z - hh]);
        pts.push([hw, -0.01, z - hh]);
        pts.push([hw, -0.01, z + hh]);
        pts.push([-hw, -0.01, z + hh]);
        pts.push([-hw, 0.01, z - hh]);
        pts.push([hw, 0.01, z - hh]);
        pts.push([hw, 0.01, z + hh]);
        pts.push([-hw, 0.01, z + hh]);
        // Front face
        polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 2) as i64, (bb + 3) as i64]);
        // Back face
        polys.push_cell(&[
            (bb + 4) as i64,
            (bb + 7) as i64,
            (bb + 6) as i64,
            (bb + 5) as i64,
        ]);
        // Ribbon connections to next block
        if b + 1 < nb {
            let nz = -spacing * (b + 1) as f64;
            // Left ribbon
            let r0 = pts.len();
            pts.push([-hw, -0.005, z - hh]);
            let r1 = pts.len();
            pts.push([-hw, -0.005, nz + hh]);
            lines.push_cell(&[r0 as i64, r1 as i64]);
            // Right ribbon
            let r2 = pts.len();
            pts.push([hw, 0.005, z - hh]);
            let r3 = pts.len();
            pts.push([hw, 0.005, nz + hh]);
            lines.push_cell(&[r2 as i64, r3 as i64]);
        }
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
    fn test_jacobs() {
        let m = jacobs_ladder(2.0, 1.0, 5);
        assert!(m.points.len() > 40);
        assert!(m.polys.num_cells() >= 10);
        assert!(m.lines.num_cells() >= 8);
    }
}
