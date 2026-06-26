//! Advanced mesh measurements: surface integral, volume, moments.

use crate::data::PolyData;

/// Compute the signed volume of a closed triangle mesh.
pub fn signed_volume(mesh: &PolyData) -> f64 {
    let mut vol = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            vol += a[0] * (b[1] * c[2] - b[2] * c[1])
                + b[0] * (c[1] * a[2] - c[2] * a[1])
                + c[0] * (a[1] * b[2] - a[2] * b[1]);
        }
    }
    vol / 6.0
}

/// Compute the total surface area of a mesh.
pub fn surface_area(mesh: &PolyData) -> f64 {
    let mut area = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            area += tri_area(a, b, c);
        }
    }
    area
}

/// Compute center of mass of a surface mesh (area-weighted centroid).
pub fn center_of_mass(mesh: &PolyData) -> [f64; 3] {
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    let mut total = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell(cell, mesh.points.len()) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let area = tri_area(a, b, c);
            let mx = (a[0] + b[0] + c[0]) / 3.0;
            let my = (a[1] + b[1] + c[1]) / 3.0;
            let mz = (a[2] + b[2] + c[2]) / 3.0;
            cx += area * mx;
            cy += area * my;
            cz += area * mz;
            total += area;
        }
    }
    if total > 1e-15 {
        [cx / total, cy / total, cz / total]
    } else {
        [0.0; 3]
    }
}

fn tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (nx * nx + ny * ny + nz * nz).sqrt()
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
}

/// Compute the moment of inertia tensor (3x3 matrix).
pub fn inertia_tensor(mesh: &PolyData) -> [[f64; 3]; 3] {
    let n = mesh.points.len();
    let mut tensor = [[0.0; 3]; 3];
    let com = center_of_mass(mesh);
    for i in 0..n {
        let p = mesh.points.get(i);
        let d = [p[0] - com[0], p[1] - com[1], p[2] - com[2]];
        let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        tensor[0][0] += r2 - d[0] * d[0];
        tensor[0][1] -= d[0] * d[1];
        tensor[0][2] -= d[0] * d[2];
        tensor[1][0] -= d[1] * d[0];
        tensor[1][1] += r2 - d[1] * d[1];
        tensor[1][2] -= d[1] * d[2];
        tensor[2][0] -= d[2] * d[0];
        tensor[2][1] -= d[2] * d[1];
        tensor[2][2] += r2 - d[2] * d[2];
    }
    tensor
}

/// Compute sphericity: how close the shape is to a sphere.
/// 1.0 = perfect sphere, 0.0 = very non-spherical.
pub fn sphericity(mesh: &PolyData) -> f64 {
    let vol = signed_volume(mesh).abs();
    let area = surface_area(mesh);
    if area < 1e-15 {
        return 0.0;
    }
    let pi = std::f64::consts::PI;
    (pi.powf(1.0 / 3.0) * (6.0 * vol).powf(2.0 / 3.0)) / area
}

/// Compute compactness (isoperimetric ratio): V² / A³.
pub fn compactness(mesh: &PolyData) -> f64 {
    let vol = signed_volume(mesh).abs();
    let area = surface_area(mesh);
    if area < 1e-15 {
        return 0.0;
    }
    (36.0 * std::f64::consts::PI * vol * vol) / (area * area * area)
}

/// Comprehensive measurement summary.
#[derive(Debug, Clone)]
pub struct MeshMeasurements {
    pub volume: f64,
    pub surface_area: f64,
    pub center_of_mass: [f64; 3],
    pub sphericity: f64,
    pub compactness: f64,
}

impl std::fmt::Display for MeshMeasurements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Volume={:.6}, Area={:.6}, CoM=[{:.4},{:.4},{:.4}], Sphericity={:.4}, Compactness={:.4}",
            self.volume, self.surface_area, self.center_of_mass[0], self.center_of_mass[1], self.center_of_mass[2],
            self.sphericity, self.compactness)
    }
}

pub fn measure_all(mesh: &PolyData) -> MeshMeasurements {
    MeshMeasurements {
        volume: signed_volume(mesh).abs(),
        surface_area: surface_area(mesh),
        center_of_mass: center_of_mass(mesh),
        sphericity: sphericity(mesh),
        compactness: compactness(mesh),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unit_cube_volume() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
            vec![
                [0, 2, 1],
                [0, 3, 2],
                [4, 5, 6],
                [4, 6, 7],
                [0, 1, 5],
                [0, 5, 4],
                [2, 3, 7],
                [2, 7, 6],
                [0, 4, 7],
                [0, 7, 3],
                [1, 2, 6],
                [1, 6, 5],
            ],
        );
        let vol = signed_volume(&mesh).abs();
        assert!((vol - 1.0).abs() < 0.01, "vol={vol}");
    }
    #[test]
    fn area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((surface_area(&mesh) - 0.5).abs() < 0.01);
    }
    #[test]
    fn com() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let c = center_of_mass(&mesh);
        assert!((c[0] - 1.0).abs() < 0.01);
    }
    #[test]
    fn sphere_sphericity() {
        let mesh = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams {
                radius: 1.0,
                ..Default::default()
            },
        );
        let s = sphericity(&mesh);
        assert!(s > 0.8, "sphericity={s}");
    }
    #[test]
    fn display() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let m = measure_all(&mesh);
        let s = format!("{m}");
        assert!(s.contains("Volume="));
    }
}
