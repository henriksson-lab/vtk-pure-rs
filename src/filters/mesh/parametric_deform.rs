//! Parametric mesh deformations: FFD (free-form deformation), lattice deform.

use crate::data::{Points, PolyData};

/// Sine-wave deformation along an axis.
pub fn sine_deform(
    mesh: &PolyData,
    axis: usize,
    amplitude: f64,
    wavelength: f64,
    deform_axis: usize,
) -> PolyData {
    let n = mesh.points.len();
    let mut new_pts = Points::<f64>::new();
    for i in 0..n {
        let mut p = mesh.points.get(i);
        let phase = 2.0 * std::f64::consts::PI * p[axis] / wavelength;
        p[deform_axis] += amplitude * phase.sin();
        new_pts.push(p);
    }
    let mut result = mesh.clone();
    result.points = new_pts;
    result
}

/// Taper deformation: linearly scale cross-section along an axis.
pub fn taper_deform(mesh: &PolyData, axis: usize, start_scale: f64, end_scale: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }

    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        let v = mesh.points.get(i)[axis];
        min_v = min_v.min(v);
        max_v = max_v.max(v);
    }
    let range = (max_v - min_v).max(1e-15);

    // Compute centroid of cross-section
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    let center = [cx / nf, cy / nf, cz / nf];

    let mut new_pts = Points::<f64>::new();
    for i in 0..n {
        let mut p = mesh.points.get(i);
        let t = (p[axis] - min_v) / range;
        let scale = start_scale + t * (end_scale - start_scale);
        // Scale perpendicular to axis around center
        for j in 0..3 {
            if j != axis {
                p[j] = center[j] + (p[j] - center[j]) * scale;
            }
        }
        new_pts.push(p);
    }
    let mut result = mesh.clone();
    result.points = new_pts;
    result
}

/// Spherize deformation: push points toward a sphere surface.
pub fn spherize_deform(mesh: &PolyData, center: [f64; 3], radius: f64, factor: f64) -> PolyData {
    let n = mesh.points.len();
    let mut new_pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let dx = p[0] - center[0];
        let dy = p[1] - center[1];
        let dz = p[2] - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < 1e-15 {
            new_pts.push(p);
            continue;
        }
        let target_dist = radius;
        let new_dist = dist + factor * (target_dist - dist);
        let scale = new_dist / dist;
        new_pts.push([
            center[0] + dx * scale,
            center[1] + dy * scale,
            center[2] + dz * scale,
        ]);
    }
    let mut result = mesh.clone();
    result.points = new_pts;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = sine_deform(&mesh, 0, 0.5, 2.0, 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn taper() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 1.0, 0.0]]);
        let result = taper_deform(&mesh, 1, 1.0, 2.0);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn spherize() {
        let mesh = PolyData::from_points(vec![[0.5, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 0.0, 0.5]]);
        let result = spherize_deform(&mesh, [0.0, 0.0, 0.0], 1.0, 0.5);
        // Points should be pushed toward radius 1.0
        for i in 0..3 {
            let p = result.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(r > 0.5 && r <= 1.0, "r={r}");
        }
    }
}
