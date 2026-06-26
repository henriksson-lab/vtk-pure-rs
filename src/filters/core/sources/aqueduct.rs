//! Roman aqueduct (series of arches) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn aqueduct(
    num_arches: usize,
    arch_span: f64,
    arch_height: f64,
    pier_width: f64,
    total_height: f64,
    thickness: f64,
    resolution: usize,
) -> PolyData {
    let na = num_arches.max(1);
    let res = resolution.max(4);
    let ht = thickness / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let bay = arch_span + pier_width;
    // Piers
    let add_box = |pts: &mut Points<f64>,
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
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    for pi in 0..=na {
        let x = pi as f64 * bay;
        add_box(
            &mut pts,
            &mut polys,
            x,
            -ht,
            0.0,
            x + pier_width,
            ht,
            total_height,
        );
    }
    // Arches
    for ai in 0..na {
        let x_start = ai as f64 * bay + pier_width;
        let x_center = x_start + arch_span / 2.0;
        let arch_r = arch_span / 2.0;
        for i in 0..res {
            let a0 = std::f64::consts::PI * i as f64 / res as f64;
            let a1 = std::f64::consts::PI * (i + 1) as f64 / res as f64;
            let b = pts.len();
            pts.push([
                x_center + arch_r * a0.cos(),
                -ht,
                arch_height - arch_r + arch_r * a0.sin(),
            ]);
            pts.push([
                x_center + arch_r * a1.cos(),
                -ht,
                arch_height - arch_r + arch_r * a1.sin(),
            ]);
            pts.push([
                x_center + arch_r * a1.cos(),
                ht,
                arch_height - arch_r + arch_r * a1.sin(),
            ]);
            pts.push([
                x_center + arch_r * a0.cos(),
                ht,
                arch_height - arch_r + arch_r * a0.sin(),
            ]);
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
        }
    }
    // Top channel
    let total_length = na as f64 * bay + pier_width;
    add_box(
        &mut pts,
        &mut polys,
        0.0,
        -ht * 1.5,
        total_height,
        total_length,
        ht * 1.5,
        total_height + thickness,
    );
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let a = aqueduct(3, 4.0, 5.0, 1.0, 8.0, 0.5, 6);
        assert!(a.polys.num_cells() > 30);
    }
}
