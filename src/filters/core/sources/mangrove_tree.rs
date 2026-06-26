//! Mangrove tree with aerial root system.
use crate::data::{CellArray, Points, PolyData};

pub fn mangrove_tree(trunk_height: f64, canopy_radius: f64, n_roots: usize) -> PolyData {
    let nr = n_roots.max(4);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Trunk
    let t0 = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let t1 = pts.len();
    pts.push([0.0, 0.0, trunk_height]);
    lines.push_cell(&[t0 as i64, t1 as i64]);
    // Aerial roots (arching lines from trunk to ground)
    for i in 0..nr {
        let a = 2.0 * std::f64::consts::PI * i as f64 / nr as f64;
        let root_r = trunk_height * 0.5;
        let mid = pts.len();
        pts.push([
            root_r * 0.3 * a.cos(),
            root_r * 0.3 * a.sin(),
            trunk_height * 0.3,
        ]);
        let tip = pts.len();
        pts.push([root_r * a.cos(), root_r * a.sin(), 0.0]);
        lines.push_cell(&[t0 as i64, mid as i64]);
        lines.push_cell(&[mid as i64, tip as i64]);
    }
    // Canopy (simple sphere of triangles)
    let na = 8;
    let np = 3;
    let canopy_z = trunk_height + canopy_radius * 0.5;
    let top = pts.len();
    pts.push([0.0, 0.0, canopy_z + canopy_radius]);
    for p in 1..=np {
        let phi = std::f64::consts::PI * p as f64 / (np + 1) as f64;
        let r = canopy_radius * phi.sin();
        let z = canopy_z + canopy_radius * phi.cos();
        for j in 0..na {
            let th = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * th.cos(), r * th.sin(), z]);
        }
    }
    for j in 0..na {
        polys.push_cell(&[
            top as i64,
            (top + 1 + j) as i64,
            (top + 1 + (j + 1) % na) as i64,
        ]);
    }
    for p in 0..(np - 1) {
        let b0 = top + 1 + p * na;
        let b1 = top + 1 + (p + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    let bottom = pts.len();
    pts.push([0.0, 0.0, canopy_z - canopy_radius]);
    let bottom_ring = top + 1 + (np - 1) * na;
    for j in 0..na {
        polys.push_cell(&[
            bottom as i64,
            (bottom_ring + (j + 1) % na) as i64,
            (bottom_ring + j) as i64,
        ]);
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
    fn test_mangrove() {
        let m = mangrove_tree(5.0, 2.0, 6);
        assert!(m.points.len() > 30);
        assert!(m.lines.num_cells() > 10);
        assert!(m.polys.num_cells() > 10);
    }
}
