//! Digital metronome with display and controls.
use crate::data::{CellArray, Points, PolyData};

pub fn metronome_digital(width: f64, height: f64, depth: f64, bpm: usize) -> PolyData {
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();

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

    // Main case.
    add_box(&mut pts, &mut polys, -hw, -hd, 0.0, hw, hd, height);

    // Speaker grille.
    let grille_z = height * 0.18;
    for i in 0..5 {
        let x = -hw * 0.45 + i as f64 * width * 0.225;
        let b = pts.len();
        pts.push([x, -hd - depth * 0.01, grille_z]);
        pts.push([x, -hd - depth * 0.01, grille_z + height * 0.18]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }

    // Seven-segment BPM display.
    let digits = [
        ((bpm / 100) % 10) as u8,
        ((bpm / 10) % 10) as u8,
        (bpm % 10) as u8,
    ];
    let patterns: [u8; 10] = [
        0b1111110, 0b0110000, 0b1101101, 0b1111001, 0b0110011, 0b1011011, 0b1011111, 0b1110000,
        0b1111111, 0b1111011,
    ];
    let digit_w = width * 0.18;
    let digit_h = height * 0.28;
    let seg_t = digit_w * 0.18;
    let display_z = height * 0.58;
    for (di, digit) in digits.iter().enumerate() {
        let x0 = -digit_w * 1.75 + di as f64 * digit_w * 1.75;
        let y = -hd - depth * 0.02;
        let pattern = patterns[*digit as usize];
        let segments = [
            (
                x0 + seg_t,
                display_z + digit_h,
                x0 + digit_w - seg_t,
                display_z + digit_h,
            ),
            (
                x0 + digit_w,
                display_z + digit_h * 0.5 + seg_t,
                x0 + digit_w,
                display_z + digit_h - seg_t,
            ),
            (
                x0 + digit_w,
                display_z + seg_t,
                x0 + digit_w,
                display_z + digit_h * 0.5 - seg_t,
            ),
            (x0 + seg_t, display_z, x0 + digit_w - seg_t, display_z),
            (x0, display_z + seg_t, x0, display_z + digit_h * 0.5 - seg_t),
            (
                x0,
                display_z + digit_h * 0.5 + seg_t,
                x0,
                display_z + digit_h - seg_t,
            ),
            (
                x0 + seg_t,
                display_z + digit_h * 0.5,
                x0 + digit_w - seg_t,
                display_z + digit_h * 0.5,
            ),
        ];

        for (si, &(sx0, sz0, sx1, sz1)) in segments.iter().enumerate() {
            if pattern & (1 << (6 - si)) == 0 {
                continue;
            }
            let b = pts.len();
            if (sz0 - sz1).abs() < 1e-12 {
                pts.push([sx0, y, sz0 - seg_t * 0.5]);
                pts.push([sx1, y, sz1 - seg_t * 0.5]);
                pts.push([sx1, y, sz1 + seg_t * 0.5]);
                pts.push([sx0, y, sz0 + seg_t * 0.5]);
            } else {
                pts.push([sx0 - seg_t * 0.5, y, sz0]);
                pts.push([sx0 + seg_t * 0.5, y, sz0]);
                pts.push([sx1 + seg_t * 0.5, y, sz1]);
                pts.push([sx1 - seg_t * 0.5, y, sz1]);
            }
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
        }
    }

    // Control buttons.
    let button_w = width * 0.16;
    let button_h = height * 0.06;
    for i in 0..3 {
        let x = -button_w * 1.4 + i as f64 * button_w * 1.4;
        add_box(
            &mut pts,
            &mut polys,
            x - button_w * 0.5,
            -hd - depth * 0.03,
            height * 0.38,
            x + button_w * 0.5,
            -hd,
            height * 0.38 + button_h,
        );
    }

    // Beat indicator.
    let center = pts.len();
    let indicator_r = width * 0.05;
    pts.push([0.0, -hd - depth * 0.04, height * 0.88]);
    for i in 0..12 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 12.0;
        pts.push([
            indicator_r * a.cos(),
            -hd - depth * 0.04,
            height * 0.88 + indicator_r * a.sin(),
        ]);
    }
    for i in 0..12 {
        polys.push_cell(&[
            center as i64,
            (center + 1 + i) as i64,
            (center + 1 + (i + 1) % 12) as i64,
        ]);
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
    fn test_metronome_digital() {
        let m = metronome_digital(3.0, 5.0, 0.5, 120);
        assert!(m.points.len() > 40);
        assert!(m.polys.num_cells() > 20);
        assert!(m.lines.num_cells() > 0);
    }
}
