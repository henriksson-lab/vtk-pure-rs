//! Neuron (nerve cell) geometry with soma, dendrites, and axon.
use crate::data::{CellArray, Points, PolyData};
pub fn neuron(
    soma_r: f64,
    num_dendrites: usize,
    dendrite_length: f64,
    axon_length: f64,
    axon_branches: usize,
    resolution: usize,
) -> PolyData {
    let _res = resolution.max(6);
    let nd = num_dendrites.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Soma (sphere approximation)
    let sb = pts.len();
    pts.push([soma_r, 0.0, 0.0]);
    pts.push([-soma_r, 0.0, 0.0]);
    pts.push([0.0, soma_r, 0.0]);
    pts.push([0.0, -soma_r, 0.0]);
    pts.push([0.0, 0.0, soma_r]);
    pts.push([0.0, 0.0, -soma_r]);
    let faces = [
        [0, 2, 4],
        [2, 1, 4],
        [1, 3, 4],
        [3, 0, 4],
        [0, 5, 2],
        [2, 5, 1],
        [1, 5, 3],
        [3, 5, 0],
    ];
    for f in &faces {
        polys.push_cell(&[(sb + f[0]) as i64, (sb + f[1]) as i64, (sb + f[2]) as i64]);
    }
    // Dendrites (branching lines)
    let mut rng = 12345u64;
    for di in 0..nd {
        let a = 2.0 * std::f64::consts::PI * di as f64 / nd as f64;
        let dir = [a.cos() * 0.7, a.sin() * 0.7, 0.3];
        let d = normalize(dir);
        // Main dendrite
        let steps = 5;
        let seg_l = dendrite_length / steps as f64;
        let mut cur = [d[0] * soma_r, d[1] * soma_r, d[2] * soma_r];
        for si in 0..steps {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let jx = ((rng >> 33) as f64 / u32::MAX as f64 - 0.5) * 0.3;
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let jy = ((rng >> 33) as f64 / u32::MAX as f64 - 0.5) * 0.3;
            let next = [
                cur[0] + d[0] * seg_l + jx,
                cur[1] + d[1] * seg_l + jy,
                cur[2] + d[2] * seg_l,
            ];
            let lb = pts.len();
            pts.push(cur);
            pts.push(next);
            lines.push_cell(&[lb as i64, (lb + 1) as i64]);
            // Small branches
            if si > 1 {
                rng = rng
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let ba = ((rng >> 33) as f64 / u32::MAX as f64) * std::f64::consts::PI;
                let branch_end = [
                    cur[0] + seg_l * 0.5 * ba.cos(),
                    cur[1] + seg_l * 0.5 * ba.sin(),
                    cur[2],
                ];
                let bb = pts.len();
                pts.push(cur);
                pts.push(branch_end);
                lines.push_cell(&[bb as i64, (bb + 1) as i64]);
            }
            cur = next;
        }
    }
    // Axon (long line going down with branches at end)
    let axon_dir = [0.0, 0.0, -1.0];
    let axon_start = [0.0, 0.0, -soma_r];
    let axon_steps = 8;
    let axon_seg = axon_length / axon_steps as f64;
    let mut cur = axon_start;
    for _ in 0..axon_steps {
        let next = [cur[0], cur[1], cur[2] + axon_dir[2] * axon_seg];
        let lb = pts.len();
        pts.push(cur);
        pts.push(next);
        lines.push_cell(&[lb as i64, (lb + 1) as i64]);
        cur = next;
    }
    // Axon terminal branches
    for bi in 0..axon_branches {
        let ba = 2.0 * std::f64::consts::PI * bi as f64 / axon_branches as f64;
        let branch_end = [
            cur[0] + axon_length * 0.1 * ba.cos(),
            cur[1] + axon_length * 0.1 * ba.sin(),
            cur[2] - axon_length * 0.05,
        ];
        let bb = pts.len();
        pts.push(cur);
        pts.push(branch_end);
        lines.push_cell(&[bb as i64, (bb + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let n = neuron(1.0, 5, 4.0, 8.0, 4, 8);
        assert!(n.polys.num_cells() >= 8);
        assert!(n.lines.num_cells() > 20);
    }
}
