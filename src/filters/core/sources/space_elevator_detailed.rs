//! Detailed space elevator with counterweight, stations, and climbers.
use crate::data::{CellArray, Points, PolyData};
pub fn detailed_space_elevator(
    base_r: f64,
    geo_alt: f64,
    counter_alt: f64,
    num_stations: usize,
    num_climbers: usize,
) -> PolyData {
    let ns = num_stations.max(1);
    let nc = num_climbers.max(0);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let ab = |pts: &mut Points<f64>,
              polys: &mut CellArray,
              x0: f64,
              y0: f64,
              z0: f64,
              x1: f64,
              y1: f64,
              z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    };
    // Tether (multiple cables)
    for &offset in &[-base_r * 0.001, base_r * 0.001] {
        let mut ids = Vec::new();
        let segs = 30;
        for i in 0..=segs {
            let z = counter_alt * i as f64 / segs as f64;
            let idx = pts.len();
            pts.push([offset, 0.0, z]);
            ids.push(idx as i64);
        }
        lines.push_cell(&ids);
    }
    // Base anchor platform
    let pr = base_r * 0.005;
    ab(&mut pts, &mut polys, -pr, -pr, 0.0, pr, pr, base_r * 0.001);
    // Stations along tether
    for si in 0..ns {
        let z = geo_alt * (si + 1) as f64 / (ns + 1) as f64;
        let ss = base_r * 0.003;
        ab(&mut pts, &mut polys, -ss, -ss, z - ss, ss, ss, z + ss);
    }
    // GEO station (larger)
    let gs = base_r * 0.008;
    ab(
        &mut pts,
        &mut polys,
        -gs,
        -gs,
        geo_alt - gs,
        gs,
        gs,
        geo_alt + gs,
    );
    // Counterweight
    let cw = base_r * 0.012;
    ab(
        &mut pts,
        &mut polys,
        -cw,
        -cw,
        counter_alt - cw,
        cw,
        cw,
        counter_alt + cw,
    );
    // Climbers
    for ci in 0..nc {
        let z = geo_alt * 0.1 + ci as f64 * geo_alt * 0.3;
        let cs = base_r * 0.002;
        ab(&mut pts, &mut polys, -cs, -cs, z - cs, cs, cs, z + cs);
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
        let e = detailed_space_elevator(6371.0, 35786.0, 50000.0, 3, 2);
        assert!(e.polys.num_cells() > 20);
        assert!(e.lines.num_cells() >= 2);
    }
}
