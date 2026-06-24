//! Japanese Torii gate.
use crate::data::{CellArray, Points, PolyData};

pub fn torii_gate(width: f64, height: f64, pillar_radius: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Two pillars (cylinders)
    for side in [-1.0f64, 1.0] {
        let cx = side * hw;
        for s in 0..=1 {
            let z = height * s as f64;
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([cx + pillar_radius * a.cos(), pillar_radius * a.sin(), z]);
            }
        }
    }
    // Pillar faces
    for pillar in 0..2 {
        let b = pillar * na * 2;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[
                (b + j) as i64,
                (b + na + j) as i64,
                (b + na + j1) as i64,
                (b + j1) as i64,
            ]);
        }
    }
    // Kasagi (top beam, wider than pillars) - extends beyond
    let overhang = width * 0.15;
    let beam_h = height * 0.05;
    let bb = pts.len();
    pts.push([-hw - overhang, -pillar_radius * 2.0, height]);
    pts.push([hw + overhang, -pillar_radius * 2.0, height]);
    pts.push([hw + overhang, pillar_radius * 2.0, height]);
    pts.push([-hw - overhang, pillar_radius * 2.0, height]);
    pts.push([-hw - overhang, -pillar_radius * 2.0, height + beam_h]);
    pts.push([hw + overhang, -pillar_radius * 2.0, height + beam_h]);
    pts.push([hw + overhang, pillar_radius * 2.0, height + beam_h]);
    pts.push([-hw - overhang, pillar_radius * 2.0, height + beam_h]);
    // Top and front faces
    polys.push_cell(&[
        (bb + 4) as i64,
        (bb + 5) as i64,
        (bb + 6) as i64,
        (bb + 7) as i64,
    ]); // top
    polys.push_cell(&[bb as i64, (bb + 1) as i64, (bb + 5) as i64, (bb + 4) as i64]); // front
    polys.push_cell(&[
        (bb + 2) as i64,
        (bb + 3) as i64,
        (bb + 7) as i64,
        (bb + 6) as i64,
    ]); // back
        // Nuki (cross beam, lower)
    let nuki_z = height * 0.75;
    let nb = pts.len();
    pts.push([-hw, -pillar_radius, nuki_z]);
    pts.push([hw, -pillar_radius, nuki_z]);
    pts.push([hw, pillar_radius, nuki_z]);
    pts.push([-hw, pillar_radius, nuki_z]);
    pts.push([-hw, -pillar_radius, nuki_z + beam_h * 0.7]);
    pts.push([hw, -pillar_radius, nuki_z + beam_h * 0.7]);
    pts.push([hw, pillar_radius, nuki_z + beam_h * 0.7]);
    pts.push([-hw, pillar_radius, nuki_z + beam_h * 0.7]);
    polys.push_cell(&[
        (nb + 4) as i64,
        (nb + 5) as i64,
        (nb + 6) as i64,
        (nb + 7) as i64,
    ]);
    polys.push_cell(&[nb as i64, (nb + 1) as i64, (nb + 5) as i64, (nb + 4) as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_torii() {
        let m = torii_gate(4.0, 5.0, 0.2, 8);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 10);
    }
}
