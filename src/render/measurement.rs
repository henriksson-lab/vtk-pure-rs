use crate::data::PolyData;

/// Measurement results for a PolyData mesh.
#[derive(Debug, Clone)]
pub struct MeshMeasurements {
    /// Total surface area (sum of triangle areas).
    pub surface_area: f64,
    /// Total edge length.
    pub total_edge_length: f64,
    /// Average edge length.
    pub avg_edge_length: f64,
    /// Minimum edge length.
    pub min_edge_length: f64,
    /// Maximum edge length.
    pub max_edge_length: f64,
    /// Number of edges.
    pub num_edges: usize,
    /// Bounding box diagonal.
    pub diagonal: f64,
}

/// Compute geometric measurements of a PolyData mesh.
pub fn measure(pd: &PolyData) -> MeshMeasurements {
    let mut surface_area = 0.0;
    let mut edges = std::collections::HashSet::new();
    let mut total_edge_len = 0.0;
    let mut min_edge = f64::INFINITY;
    let mut max_edge = 0.0f64;

    for cell in pd.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Triangle fan area
        let p0 = pd.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = pd.points.get(cell[i] as usize);
            let p2 = pd.points.get(cell[i + 1] as usize);
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            surface_area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }

        // Edge lengths
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            if edges.insert(key) {
                let pa = pd.points.get(a);
                let pb = pd.points.get(b);
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                let dz = pb[2] - pa[2];
                let len = (dx * dx + dy * dy + dz * dz).sqrt();
                total_edge_len += len;
                min_edge = min_edge.min(len);
                max_edge = max_edge.max(len);
            }
        }
    }

    let num_edges = edges.len();
    let avg = if num_edges > 0 {
        total_edge_len / num_edges as f64
    } else {
        0.0
    };
    let bb = pd.points.bounds();

    MeshMeasurements {
        surface_area,
        total_edge_length: total_edge_len,
        avg_edge_length: avg,
        min_edge_length: if min_edge.is_finite() { min_edge } else { 0.0 },
        max_edge_length: max_edge,
        num_edges,
        diagonal: bb.diagonal_length(),
    }
}

/// Compute distance between two points.
pub fn point_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute angle (in degrees) between three points (angle at vertex b).
pub fn angle_at_vertex(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let ba = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
    let dot = ba[0] * bc[0] + ba[1] * bc[1] + ba[2] * bc[2];
    let mag_ba = (ba[0] * ba[0] + ba[1] * ba[1] + ba[2] * ba[2]).sqrt();
    let mag_bc = (bc[0] * bc[0] + bc[1] * bc[1] + bc[2] * bc[2]).sqrt();
    if mag_ba < 1e-15 || mag_bc < 1e-15 {
        return 0.0;
    }
    let cos_angle = (dot / (mag_ba * mag_bc)).clamp(-1.0, 1.0);
    cos_angle.acos().to_degrees()
}

/// Compute area of a triangle given three vertices.
pub fn triangle_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

impl std::fmt::Display for MeshMeasurements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "area={:.4}, edges={} (len: {:.4}..{:.4}, avg={:.4}), diag={:.4}",
            self.surface_area,
            self.num_edges,
            self.min_edge_length,
            self.max_edge_length,
            self.avg_edge_length,
            self.diagonal
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_triangle_area() {
        let a = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 0.5).abs() < 1e-10);
    }

    #[test]
    fn right_angle() {
        let angle = angle_at_vertex([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - 90.0).abs() < 1e-6);
    }

    #[test]
    fn point_dist() {
        let d = point_distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn measure_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let m = measure(&pd);
        assert!((m.surface_area - 0.5).abs() < 1e-10);
        assert_eq!(m.num_edges, 3);
        assert!(m.min_edge_length > 0.0);
        assert!(m.max_edge_length >= m.min_edge_length);
    }

    #[test]
    fn measure_display() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let m = measure(&pd);
        let s = format!("{m}");
        assert!(s.contains("area="));
        assert!(s.contains("edges="));
    }
}
