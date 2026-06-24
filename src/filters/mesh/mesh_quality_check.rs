//! Mesh quality checks and metrics.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Quality metrics for a mesh.
pub struct QualityReport {
    pub num_degenerate: usize,
    pub num_inverted: usize,
    pub min_angle_deg: f64,
    pub max_angle_deg: f64,
    pub min_area: f64,
    pub max_area: f64,
    pub avg_quality: f64,
}

/// Compute quality report for a triangle mesh.
pub fn quality_report(mesh: &PolyData) -> QualityReport {
    let mut num_degen = 0;
    let mut num_inv = 0;
    let mut min_angle = 180.0f64;
    let mut max_angle = 0.0f64;
    let mut min_area = f64::INFINITY;
    let mut max_area = 0.0f64;
    let mut quality_sum = 0.0;
    let mut count = 0;

    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let area = tri_area(a, b, c);
        if area < 1e-15 {
            num_degen += 1;
            continue;
        }

        let angles = tri_angles(a, b, c);
        for &ang in &angles {
            min_angle = min_angle.min(ang);
            max_angle = max_angle.max(ang);
        }
        min_area = min_area.min(area);
        max_area = max_area.max(area);

        // Quality = min_angle / 60 (ideal for equilateral)
        let q = angles.iter().cloned().fold(f64::INFINITY, f64::min) / 60.0;
        quality_sum += q;
        count += 1;

        // Check for inverted (negative area in 2D sense)
        let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if cross < 0.0 {
            num_inv += 1;
        }
    }

    QualityReport {
        num_degenerate: num_degen,
        num_inverted: num_inv,
        min_angle_deg: min_angle,
        max_angle_deg: max_angle,
        min_area: if min_area.is_infinite() {
            0.0
        } else {
            min_area
        },
        max_area,
        avg_quality: if count > 0 {
            quality_sum / count as f64
        } else {
            0.0
        },
    }
}

/// Attach per-triangle quality as cell data.
pub fn attach_quality(mesh: &PolyData) -> PolyData {
    let mut qualities = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            qualities.push(0.0);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let angles = tri_angles(a, b, c);
        let q = angles.iter().cloned().fold(f64::INFINITY, f64::min) / 60.0;
        qualities.push(q.clamp(0.0, 1.0));
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Quality", qualities, 1,
        )));
    result
}

fn tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

fn tri_angles(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let angle_at = |p: [f64; 3], q: [f64; 3], r: [f64; 3]| -> f64 {
        let v1 = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        let v2 = [r[0] - p[0], r[1] - p[1], r[2] - p[2]];
        let d = v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2];
        let l1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
        let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
        if l1 < 1e-15 || l2 < 1e-15 {
            return 0.0;
        }
        (d / (l1 * l2)).clamp(-1.0, 1.0).acos().to_degrees()
    };
    [angle_at(a, b, c), angle_at(b, c, a), angle_at(c, a, b)]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equilateral() {
        let h = 3.0f64.sqrt() / 2.0;
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = quality_report(&mesh);
        assert!((r.min_angle_deg - 60.0).abs() < 1.0);
        assert!((r.avg_quality - 1.0).abs() < 0.1);
    }
    #[test]
    fn test_attach() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = attach_quality(&mesh);
        assert!(r.cell_data().get_array("Quality").is_some());
    }
}
