//! Triumphal arch (Arc de Triomphe-like) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn triumphal_arch(
    width: f64,
    height: f64,
    depth: f64,
    arch_width: f64,
    arch_height: f64,
    resolution: usize,
) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
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
    // Left pier
    let pier_w = (width - arch_width) / 2.0;
    add_box(
        &mut pts,
        &mut polys,
        -hw,
        -hd,
        0.0,
        -hw + pier_w,
        hd,
        height,
    );
    // Right pier
    add_box(&mut pts, &mut polys, hw - pier_w, -hd, 0.0, hw, hd, height);
    // Top entablature
    add_box(&mut pts, &mut polys, -hw, -hd, arch_height, hw, hd, height);
    // Arch (semicircular opening)
    let arch_r = arch_width / 2.0;
    let arch_cx = 0.0;
    for i in 0..res {
        let a0 = std::f64::consts::PI * i as f64 / res as f64;
        let a1 = std::f64::consts::PI * (i + 1) as f64 / res as f64;
        let ab = pts.len();
        pts.push([
            arch_cx + arch_r * a0.cos(),
            -hd,
            arch_height - arch_r + arch_r * a0.sin(),
        ]);
        pts.push([
            arch_cx + arch_r * a1.cos(),
            -hd,
            arch_height - arch_r + arch_r * a1.sin(),
        ]);
        pts.push([
            arch_cx + arch_r * a1.cos(),
            hd,
            arch_height - arch_r + arch_r * a1.sin(),
        ]);
        pts.push([
            arch_cx + arch_r * a0.cos(),
            hd,
            arch_height - arch_r + arch_r * a0.sin(),
        ]);
        polys.push_cell(&[ab as i64, (ab + 1) as i64, (ab + 2) as i64, (ab + 3) as i64]);
    }
    // Attic (top section above entablature)
    let attic_h = height * 0.15;
    add_box(
        &mut pts,
        &mut polys,
        -hw * 0.9,
        -hd * 0.9,
        height,
        hw * 0.9,
        hd * 0.9,
        height + attic_h,
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
        let a = triumphal_arch(10.0, 12.0, 3.0, 5.0, 8.0, 8);
        assert!(a.polys.num_cells() > 25);
    }
}
