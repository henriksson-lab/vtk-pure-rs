//! Helical gear with twisted teeth.
use crate::data::{CellArray, Points, PolyData};

pub fn helical_gear(
    module: f64,
    n_teeth: usize,
    face_width: f64,
    helix_angle_deg: f64,
    n_profile: usize,
    n_axial: usize,
) -> PolyData {
    let nt = n_teeth.max(6);
    let npr = n_profile.max(nt * 4);
    let nax = n_axial.max(3);
    let pitch_r = module * nt as f64 / 2.0;
    let addendum = module;
    let dedendum = module * 1.25;
    let helix = helix_angle_deg * std::f64::consts::PI / 180.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for ax in 0..=nax {
        let z = face_width * ax as f64 / nax as f64;
        let twist = z * helix.tan() / pitch_r; // twist angle at this z
        for j in 0..npr {
            let base_angle = 2.0 * std::f64::consts::PI * j as f64 / npr as f64 + twist;
            // Tooth profile: alternating addendum/dedendum
            let tooth_phase = (base_angle * nt as f64 / 2.0).sin();
            let r = pitch_r
                + if tooth_phase > 0.0 {
                    addendum * tooth_phase
                } else {
                    dedendum * tooth_phase * 0.5
                };
            pts.push([r * base_angle.cos(), r * base_angle.sin(), z]);
        }
    }
    for ax in 0..nax {
        let b0 = ax * npr;
        let b1 = (ax + 1) * npr;
        for j in 0..npr {
            let j1 = (j + 1) % npr;
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
    fn test_helical() {
        let m = helical_gear(2.0, 12, 5.0, 20.0, 48, 4);
        assert!(m.points.len() > 200);
        assert!(m.polys.num_cells() > 200);
    }
}
