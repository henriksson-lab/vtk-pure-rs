//! Periscope (vertical tube with angled mirrors at each end).
use crate::data::{CellArray, Points, PolyData};

pub fn periscope(height: f64, tube_radius: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main vertical tube
    let tube_segs = 4;
    for s in 0..=tube_segs {
        let z = height * s as f64 / tube_segs as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([tube_radius * a.cos(), tube_radius * a.sin(), z]);
        }
    }
    for s in 0..tube_segs {
        let b0 = s * na;
        let b1 = (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Top eyepiece (horizontal extension)
    let ext_len = tube_radius * 3.0;
    let top_base = pts.len();
    for s in 0..=2 {
        let y = -ext_len * s as f64 / 2.0;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([
                tube_radius * 0.7 * a.cos(),
                y,
                height + tube_radius * 0.7 * a.sin(),
            ]);
        }
    }
    for s in 0..2 {
        let b0 = top_base + s * na;
        let b1 = top_base + (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Bottom objective (horizontal extension, opposite side)
    let bot_base = pts.len();
    for s in 0..=2 {
        let y = ext_len * s as f64 / 2.0;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([tube_radius * 0.7 * a.cos(), y, tube_radius * 0.7 * a.sin()]);
        }
    }
    for s in 0..2 {
        let b0 = bot_base + s * na;
        let b1 = bot_base + (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
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
    fn test_periscope() {
        let m = periscope(5.0, 0.3, 10);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 60);
    }
}
