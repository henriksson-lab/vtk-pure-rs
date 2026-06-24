//! Helical spring/coil geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a helical spring (tube around a helix).
pub fn spring_coil(
    coil_radius: f64,
    wire_radius: f64,
    height: f64,
    turns: f64,
    tube_res: usize,
    helix_res: usize,
) -> PolyData {
    let tres = tube_res.max(3);
    let hres = (helix_res as f64 * turns).ceil() as usize;
    let hres = hres.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for ih in 0..=hres {
        let t = ih as f64 / hres as f64;
        let angle = 2.0 * std::f64::consts::PI * turns * t;
        let cx = coil_radius * angle.cos();
        let cy = coil_radius * angle.sin();
        let cz = height * t;

        // Tangent of helix
        let tx = -coil_radius * angle.sin() * 2.0 * std::f64::consts::PI * turns;
        let ty = coil_radius * angle.cos() * 2.0 * std::f64::consts::PI * turns;
        let tz = height;
        let tlen = (tx * tx + ty * ty + tz * tz).sqrt();
        let tx = tx / tlen;
        let ty = ty / tlen;
        let tz = tz / tlen;

        // Normal frame
        let (n1, n2) = perp_frame([tx, ty, tz]);

        for it in 0..tres {
            let a = 2.0 * std::f64::consts::PI * it as f64 / tres as f64;
            let c = a.cos();
            let s = a.sin();
            pts.push([
                cx + wire_radius * (c * n1[0] + s * n2[0]),
                cy + wire_radius * (c * n1[1] + s * n2[1]),
                cz + wire_radius * (c * n1[2] + s * n2[2]),
            ]);
        }
    }

    for ih in 0..hres {
        let r0 = ih * tres;
        let r1 = (ih + 1) * tres;
        for it in 0..tres {
            let it1 = (it + 1) % tres;
            polys.push_cell(&[
                (r0 + it) as i64,
                (r0 + it1) as i64,
                (r1 + it1) as i64,
                (r1 + it) as i64,
            ]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

fn perp_frame(t: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let up = if t[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let n1 = normalize(cross(t, up));
    let n2 = cross(t, n1);
    (n1, n2)
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
    fn test_spring() {
        let s = spring_coil(2.0, 0.2, 5.0, 3.0, 8, 20);
        assert!(s.points.len() > 100);
        assert!(s.polys.num_cells() > 50);
    }
    #[test]
    fn test_single_turn() {
        let s = spring_coil(1.0, 0.1, 1.0, 1.0, 6, 12);
        assert_eq!(s.points.len(), 13 * 6);
        assert_eq!(s.polys.num_cells(), 12 * 6);
    }
}
