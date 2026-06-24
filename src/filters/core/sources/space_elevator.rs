//! Space elevator concept (tether + counterweight + climber).
use crate::data::{CellArray, Points, PolyData};
pub fn space_elevator(
    base_r: f64,
    geo_altitude: f64,
    counterweight_alt: f64,
    tether_segments: usize,
) -> PolyData {
    let ns = tether_segments.max(10);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Earth surface platform
    let pr = base_r * 0.01;
    let pb = pts.len();
    pts.push([-pr, -pr, 0.0]);
    pts.push([pr, -pr, 0.0]);
    pts.push([pr, pr, 0.0]);
    pts.push([-pr, pr, 0.0]);
    polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
    // Tether
    let mut tether_ids = Vec::new();
    for i in 0..=ns {
        let t = i as f64 / ns as f64;
        let z = t * counterweight_alt;
        let idx = pts.len();
        pts.push([0.0, 0.0, z]);
        tether_ids.push(idx as i64);
    }
    lines.push_cell(&tether_ids);
    // GEO station (box at geostationary altitude)
    let gs_size = base_r * 0.005;
    let gb = pts.len();
    pts.push([-gs_size, -gs_size, geo_altitude - gs_size]);
    pts.push([gs_size, -gs_size, geo_altitude - gs_size]);
    pts.push([gs_size, gs_size, geo_altitude - gs_size]);
    pts.push([-gs_size, gs_size, geo_altitude - gs_size]);
    pts.push([-gs_size, -gs_size, geo_altitude + gs_size]);
    pts.push([gs_size, -gs_size, geo_altitude + gs_size]);
    pts.push([gs_size, gs_size, geo_altitude + gs_size]);
    pts.push([-gs_size, gs_size, geo_altitude + gs_size]);
    let f = |i: usize| (gb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    // Counterweight
    let cw = gs_size * 2.0;
    let cb = pts.len();
    pts.push([-cw, -cw, counterweight_alt - cw]);
    pts.push([cw, -cw, counterweight_alt - cw]);
    pts.push([cw, cw, counterweight_alt - cw]);
    pts.push([-cw, cw, counterweight_alt - cw]);
    pts.push([-cw, -cw, counterweight_alt + cw]);
    pts.push([cw, -cw, counterweight_alt + cw]);
    pts.push([cw, cw, counterweight_alt + cw]);
    pts.push([-cw, cw, counterweight_alt + cw]);
    let g = |i: usize| (cb + i) as i64;
    polys.push_cell(&[g(0), g(3), g(2), g(1)]);
    polys.push_cell(&[g(4), g(5), g(6), g(7)]);
    // Climber (small box on tether)
    let climber_z = geo_altitude * 0.3;
    let cs = gs_size * 0.5;
    let clb = pts.len();
    pts.push([-cs, -cs, climber_z - cs]);
    pts.push([cs, -cs, climber_z - cs]);
    pts.push([cs, cs, climber_z - cs]);
    pts.push([-cs, cs, climber_z - cs]);
    pts.push([-cs, -cs, climber_z + cs]);
    pts.push([cs, -cs, climber_z + cs]);
    pts.push([cs, cs, climber_z + cs]);
    pts.push([-cs, cs, climber_z + cs]);
    let h = |i: usize| (clb + i) as i64;
    polys.push_cell(&[h(0), h(3), h(2), h(1)]);
    polys.push_cell(&[h(4), h(5), h(6), h(7)]);
    polys.push_cell(&[h(0), h(1), h(5), h(4)]);
    polys.push_cell(&[h(2), h(3), h(7), h(6)]);
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
        let e = space_elevator(6371.0, 35786.0, 50000.0, 20);
        assert!(e.polys.num_cells() > 10);
        assert_eq!(e.lines.num_cells(), 1);
    }
}
