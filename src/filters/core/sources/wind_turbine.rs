//! Wind turbine geometry (tower + nacelle + blades).
use crate::data::{CellArray, Points, PolyData};
pub fn wind_turbine(
    tower_height: f64,
    tower_radius: f64,
    blade_length: f64,
    num_blades: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let nb = num_blades.max(2);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Tower (cylinder)
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { tower_height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([tower_radius * a.cos(), tower_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Nacelle (box)
    let nb_len = blade_length * 0.15;
    let nb_r = tower_radius * 1.5;
    let nb_start = pts.len();
    pts.push([-nb_r, -nb_r, tower_height]);
    pts.push([nb_r, -nb_r, tower_height]);
    pts.push([nb_r, nb_r, tower_height]);
    pts.push([-nb_r, nb_r, tower_height]);
    pts.push([-nb_r, -nb_r, tower_height + nb_len]);
    pts.push([nb_r, -nb_r, tower_height + nb_len]);
    pts.push([nb_r, nb_r, tower_height + nb_len]);
    pts.push([-nb_r, nb_r, tower_height + nb_len]);
    let f = |i: usize| (nb_start + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    // Blades
    let hub_z = tower_height + nb_len * 0.5;
    for bi in 0..nb {
        let angle = 2.0 * std::f64::consts::PI * bi as f64 / nb as f64;
        let dx = angle.cos() * blade_length;
        let dy = angle.sin() * blade_length;
        let b = pts.len();
        pts.push([0.0, 0.0, hub_z]);
        pts.push([dx, dy, hub_z]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
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
        let w = wind_turbine(50.0, 2.0, 25.0, 3, 8);
        assert!(w.polys.num_cells() > 5);
        assert!(w.lines.num_cells() >= 3);
    }
}
