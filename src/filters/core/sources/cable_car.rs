//! Cable car / gondola lift geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn cable_car_system(
    stations: &[[f64; 3]],
    cable_sag: f64,
    num_gondolas: usize,
    gondola_size: f64,
    resolution: usize,
) -> PolyData {
    let ns = stations.len();
    if ns < 2 {
        return PolyData::new();
    }
    let res = resolution.max(8);
    let ng = num_gondolas.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Station towers
    for s in stations {
        let tb = pts.len();
        pts.push([s[0], s[1], 0.0]);
        pts.push([s[0], s[1], s[2]]);
        lines.push_cell(&[tb as i64, (tb + 1) as i64]);
        // Tower top platform
        let tw = gondola_size;
        let pb = pts.len();
        pts.push([s[0] - tw, s[1] - tw * 0.3, s[2]]);
        pts.push([s[0] + tw, s[1] - tw * 0.3, s[2]]);
        pts.push([s[0] + tw, s[1] + tw * 0.3, s[2]]);
        pts.push([s[0] - tw, s[1] + tw * 0.3, s[2]]);
        polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
    }
    // Cables between stations
    for si in 0..ns - 1 {
        let p0 = stations[si];
        let p1 = stations[si + 1];
        let mut cable_ids = Vec::new();
        for i in 0..=res {
            let t = i as f64 / res as f64;
            let sag_f = 4.0 * t * (1.0 - t);
            let x = p0[0] + (p1[0] - p0[0]) * t;
            let y = p0[1] + (p1[1] - p0[1]) * t;
            let z = p0[2] + (p1[2] - p0[2]) * t - cable_sag * sag_f;
            let ci = pts.len();
            pts.push([x, y, z]);
            cable_ids.push(ci as i64);
        }
        lines.push_cell(&cable_ids);
    }
    // Gondolas (boxes hanging from cable)
    let _total_cable_l: f64 = (0..ns - 1)
        .map(|i| {
            let d = [
                (stations[i + 1][0] - stations[i][0]),
                (stations[i + 1][1] - stations[i][1]),
                (stations[i + 1][2] - stations[i][2]),
            ];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum();
    let gs = gondola_size / 2.0;
    for gi in 0..ng {
        let t = (gi as f64 + 0.5) / ng as f64;
        // Position along first cable segment (simplified)
        let p0 = stations[0];
        let p1 = stations[ns.min(2) - 1];
        let sag_f = 4.0 * t * (1.0 - t);
        let gx = p0[0] + (p1[0] - p0[0]) * t;
        let gy = p0[1] + (p1[1] - p0[1]) * t;
        let gz = p0[2] + (p1[2] - p0[2]) * t - cable_sag * sag_f - gs * 3.0;
        let gb = pts.len();
        pts.push([gx - gs, gy - gs, gz]);
        pts.push([gx + gs, gy - gs, gz]);
        pts.push([gx + gs, gy + gs, gz]);
        pts.push([gx - gs, gy + gs, gz]);
        pts.push([gx - gs, gy - gs, gz + gs * 2.0]);
        pts.push([gx + gs, gy - gs, gz + gs * 2.0]);
        pts.push([gx + gs, gy + gs, gz + gs * 2.0]);
        pts.push([gx - gs, gy + gs, gz + gs * 2.0]);
        let f = |i: usize| (gb + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
        // Hanger
        let hb = pts.len();
        pts.push([gx, gy, gz + gs * 2.0]);
        pts.push([gx, gy, gz + gs * 4.0]);
        lines.push_cell(&[hb as i64, (hb + 1) as i64]);
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
        let c = cable_car_system(
            &[[0.0, 0.0, 20.0], [50.0, 0.0, 30.0], [100.0, 0.0, 15.0]],
            5.0,
            4,
            1.0,
            12,
        );
        assert!(c.polys.num_cells() > 10);
        assert!(c.lines.num_cells() > 5);
    }
}
