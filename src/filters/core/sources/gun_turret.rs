//! Gun turret with rotating dome and barrel.
use crate::data::{CellArray, Points, PolyData};

pub fn gun_turret(base_radius: f64, dome_radius: f64, barrel_length: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base cylinder
    for f in 0..=1 {
        let z = base_radius * 0.3 * f as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([base_radius * a.cos(), base_radius * a.sin(), z]);
        }
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[j as i64, j1 as i64, (na + j1) as i64, (na + j) as i64]);
    }
    // Dome (hemisphere)
    let dome_z = base_radius * 0.3;
    let np = 4;
    let dome_top = pts.len();
    pts.push([0.0, 0.0, dome_z + dome_radius]);
    for p in 1..=np {
        let phi = std::f64::consts::PI / 2.0 * p as f64 / np as f64;
        let r = dome_radius * phi.sin();
        let z = dome_z + dome_radius * phi.cos();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for j in 0..na {
        polys.push_cell(&[
            dome_top as i64,
            (dome_top + 1 + j) as i64,
            (dome_top + 1 + (j + 1) % na) as i64,
        ]);
    }
    for p in 0..(np - 1) {
        let b0 = dome_top + 1 + p * na;
        let b1 = dome_top + 1 + (p + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Barrel (cylinder extending forward)
    let br = dome_radius * 0.15;
    let barrel_z = dome_z + dome_radius * 0.5;
    let bb = pts.len();
    for s in 0..=3 {
        let y = dome_radius * 0.8 + barrel_length * s as f64 / 3.0;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([br * a.cos(), y, barrel_z + br * a.sin()]);
        }
    }
    for s in 0..3 {
        let b0 = bb + s * na;
        let b1 = bb + (s + 1) * na;
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
    fn test_turret() {
        let m = gun_turret(2.0, 1.5, 4.0, 10);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 60);
    }

    #[test]
    fn dome_rings_expand_from_top_to_base() {
        let base_radius = 2.0;
        let dome_radius = 1.5;
        let na = 10;
        let m = gun_turret(base_radius, dome_radius, 4.0, na);
        let dome_top = 2 * na;
        let first_ring = dome_top + 1;
        let base_ring = dome_top + 1 + 3 * na;
        let dome_z = base_radius * 0.3;

        let first = m.points.get(first_ring);
        let base = m.points.get(base_ring);
        let first_radius = (first[0] * first[0] + first[1] * first[1]).sqrt();
        let base_radius = (base[0] * base[0] + base[1] * base[1]).sqrt();

        assert!(first_radius < base_radius);
        assert!((base_radius - dome_radius).abs() < 1e-12);
        assert!((base[2] - dome_z).abs() < 1e-12);
    }

    #[test]
    fn base_cylinder_sides_are_wound_outward() {
        let base_radius = 2.0;
        let na = 10;
        let m = gun_turret(base_radius, 1.5, 4.0, na);

        for cell_index in 0..na {
            let cell = m.polys.cell(cell_index);
            let p0 = m.points.get(cell[0] as usize);
            let p1 = m.points.get(cell[1] as usize);
            let p2 = m.points.get(cell[2] as usize);
            let normal = cross(sub(p1, p0), sub(p2, p0));
            let face_center = cell.iter().fold([0.0; 3], |mut acc, &id| {
                let p = m.points.get(id as usize);
                acc[0] += p[0] / cell.len() as f64;
                acc[1] += p[1] / cell.len() as f64;
                acc[2] += p[2] / cell.len() as f64;
                acc
            });
            assert!(dot(normal, [face_center[0], face_center[1], 0.0]) > 0.0);
        }
    }

    fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }
}
