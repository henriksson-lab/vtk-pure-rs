//! Cochlear implant (electrode array spiral).
use crate::data::{CellArray, Points, PolyData};
pub fn cochlear_implant(
    spiral_r: f64,
    turns: f64,
    num_electrodes: usize,
    electrode_r: f64,
    wire_r: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(4);
    let ne = num_electrodes.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Carrier wire (spiral)
    let total_steps = ne * 4;
    let mut wire_ids = Vec::new();
    for i in 0..=total_steps {
        let t = i as f64 / total_steps as f64;
        let a = 2.0 * std::f64::consts::PI * turns * t;
        let r = spiral_r * (1.0 - t * 0.6);
        let idx = pts.len();
        pts.push([r * a.cos(), r * a.sin(), t * spiral_r * 0.3]);
        wire_ids.push(idx as i64);
    }
    lines.push_cell(&wire_ids);
    if wire_r > 0.0 {
        let ring_start = pts.len();
        for i in 0..=total_steps {
            let t = i as f64 / total_steps as f64;
            let a = 2.0 * std::f64::consts::PI * turns * t;
            let r = spiral_r * (1.0 - t * 0.6);
            let center = [r * a.cos(), r * a.sin(), t * spiral_r * 0.3];
            for j in 0..res {
                let b = 2.0 * std::f64::consts::PI * j as f64 / res as f64;
                pts.push([
                    center[0] + wire_r * b.cos() * a.cos(),
                    center[1] + wire_r * b.cos() * a.sin(),
                    center[2] + wire_r * b.sin(),
                ]);
            }
        }
        for i in 0..total_steps {
            for j in 0..res {
                let j1 = (j + 1) % res;
                polys.push_cell(&[
                    (ring_start + i * res + j) as i64,
                    (ring_start + i * res + j1) as i64,
                    (ring_start + (i + 1) * res + j1) as i64,
                    (ring_start + (i + 1) * res + j) as i64,
                ]);
            }
        }
    }
    // Electrodes (small spheroids along the spiral)
    for ei in 0..ne {
        let t = (ei as f64 + 0.5) / ne as f64;
        let a = 2.0 * std::f64::consts::PI * turns * t;
        let r = spiral_r * (1.0 - t * 0.6);
        let ex = r * a.cos();
        let ey = r * a.sin();
        let ez = t * spiral_r * 0.3;
        let eb = pts.len();
        for iz in 0..=res {
            let phi = std::f64::consts::PI * iz as f64 / res as f64;
            let z = ez + electrode_r * phi.cos();
            let rr = electrode_r * phi.sin();
            for ia in 0..res {
                let theta = 2.0 * std::f64::consts::PI * ia as f64 / res as f64;
                pts.push([ex + rr * theta.cos(), ey + rr * theta.sin(), z]);
            }
        }
        for iz in 0..res {
            for ia in 0..res {
                let ia1 = (ia + 1) % res;
                polys.push_cell(&[
                    (eb + iz * res + ia) as i64,
                    (eb + iz * res + ia1) as i64,
                    (eb + (iz + 1) * res + ia1) as i64,
                    (eb + (iz + 1) * res + ia) as i64,
                ]);
            }
        }
    }
    // Receiver/stimulator (box at base)
    let rb = pts.len();
    let rs = spiral_r * 0.4;
    pts.push([-rs, -rs, -rs]);
    pts.push([rs, -rs, -rs]);
    pts.push([rs, rs, -rs]);
    pts.push([-rs, rs, -rs]);
    pts.push([-rs, -rs, 0.0]);
    pts.push([rs, -rs, 0.0]);
    pts.push([rs, rs, 0.0]);
    pts.push([-rs, rs, 0.0]);
    let f = |i: usize| (rb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result.lines = lines;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = cochlear_implant(0.005, 2.5, 16, 0.0005, 0.0002, 6);
        assert!(c.polys.num_cells() > 100);
        assert!(c.lines.num_cells() >= 1);
    }

    #[test]
    fn resolution_refines_geometry() {
        let coarse = cochlear_implant(0.005, 2.5, 16, 0.0005, 0.0002, 4);
        let fine = cochlear_implant(0.005, 2.5, 16, 0.0005, 0.0002, 8);
        assert!(fine.points.len() > coarse.points.len());
        assert!(fine.polys.num_cells() > coarse.polys.num_cells());
    }
}
