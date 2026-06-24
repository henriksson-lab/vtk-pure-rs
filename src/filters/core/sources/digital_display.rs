//! Seven-segment digital display digit.
use crate::data::{CellArray, Points, PolyData};

pub fn digital_display(digit: u8, size: f64) -> PolyData {
    let hs = size / 2.0;
    let sw = size * 0.15;
    let hsw = sw / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Seven segments: top, top-right, bottom-right, bottom, bottom-left, top-left, middle
    let segments: [(f64, f64, f64, f64); 7] = [
        (-hs + hsw, size, hs - hsw, size),     // top (horizontal)
        (hs - sw, hs + hsw, hs, size - hsw),   // top-right (vertical)
        (hs - sw, hsw, hs, hs - hsw),          // bottom-right (vertical)
        (-hs + hsw, 0.0, hs - hsw, 0.0),       // bottom (horizontal)
        (-hs, hsw, -hs + sw, hs - hsw),        // bottom-left (vertical)
        (-hs, hs + hsw, -hs + sw, size - hsw), // top-left (vertical)
        (-hs + hsw, hs, hs - hsw, hs),         // middle (horizontal)
    ];
    // Which segments are on for each digit
    let patterns: [u8; 10] = [
        0b1110111, 0b0010010, 0b1011101, 0b1011011, 0b0111010, 0b1101011, 0b1101111, 0b1010010,
        0b1111111, 0b1111011,
    ];
    let pattern = if (digit as usize) < 10 {
        patterns[digit as usize]
    } else {
        0
    };
    for (si, &(x0, y0, x1, y1)) in segments.iter().enumerate() {
        if pattern & (1 << (6 - si)) == 0 {
            continue;
        }
        let is_horiz = (y0 - y1).abs() < 0.01;
        let pb = pts.len();
        if is_horiz {
            pts.push([x0, 0.0, y0 - hsw]);
            pts.push([x1, 0.0, y0 - hsw]);
            pts.push([x1, 0.0, y0 + hsw]);
            pts.push([x0, 0.0, y0 + hsw]);
        } else {
            pts.push([x0 - hsw + (x1 - x0) / 2.0, 0.0, y0]);
            pts.push([x0 + hsw + (x1 - x0) / 2.0, 0.0, y0]);
            pts.push([x0 + hsw + (x1 - x0) / 2.0, 0.0, y1]);
            pts.push([x0 - hsw + (x1 - x0) / 2.0, 0.0, y1]);
        }
        polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display() {
        let m = digital_display(8, 2.0); // all 7 segments on
        assert_eq!(m.polys.num_cells(), 7);
        let m0 = digital_display(1, 2.0); // 2 segments
        assert_eq!(m0.polys.num_cells(), 2);
    }
}
