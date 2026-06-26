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
    push_box_faces(&mut polys, bb);

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
    push_box_faces(&mut polys, nb);
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

fn push_box_faces(polys: &mut CellArray, base: usize) {
    let f = |i: usize| (base + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
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
