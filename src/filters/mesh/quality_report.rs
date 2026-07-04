//! Comprehensive mesh quality report generation.

use crate::data::PolyData;

/// Comprehensive mesh quality report.
#[derive(Debug, Clone)]
pub struct MeshQualityReport {
    pub num_points: usize,
    pub num_cells: usize,
    pub num_triangles: usize,
    pub num_quads: usize,
    pub num_other: usize,
    pub num_boundary_edges: usize,
    pub num_non_manifold_edges: usize,
    pub is_closed: bool,
    pub is_manifold: bool,
    pub is_all_triangles: bool,
    pub min_area: f64,
    pub max_area: f64,
    pub mean_area: f64,
    pub min_aspect_ratio: f64,
    pub max_aspect_ratio: f64,
    pub mean_aspect_ratio: f64,
    pub min_edge_length: f64,
    pub max_edge_length: f64,
    pub surface_area: f64,
}

impl std::fmt::Display for MeshQualityReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Mesh Quality Report ===")?;
        writeln!(
            f,
            "Points: {}, Cells: {} (tri={}, quad={}, other={})",
            self.num_points, self.num_cells, self.num_triangles, self.num_quads, self.num_other
        )?;
        writeln!(
            f,
            "Closed: {}, Manifold: {}, All triangles: {}",
            self.is_closed, self.is_manifold, self.is_all_triangles
        )?;
        writeln!(
            f,
            "Boundary edges: {}, Non-manifold edges: {}",
            self.num_boundary_edges, self.num_non_manifold_edges
        )?;
        writeln!(
            f,
            "Area: min={:.6}, max={:.6}, mean={:.6}, total={:.6}",
            self.min_area, self.max_area, self.mean_area, self.surface_area
        )?;
        writeln!(
            f,
            "Aspect ratio: min={:.4}, max={:.4}, mean={:.4}",
            self.min_aspect_ratio, self.max_aspect_ratio, self.mean_aspect_ratio
        )?;
        write!(
            f,
            "Edge length: min={:.6}, max={:.6}",
            self.min_edge_length, self.max_edge_length
        )
    }
}

/// Generate a comprehensive quality report for a mesh.
pub fn mesh_quality_report(mesh: &PolyData) -> MeshQualityReport {
    let n_pts = mesh.points.len();
    let mut n_tri = 0usize;
    let mut n_quad = 0usize;
    let mut n_other = 0usize;

    let mut areas: Vec<f64> = Vec::new();
    let mut aspect_ratios: Vec<f64> = Vec::new();
    let mut edge_set: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let mut min_edge = f64::MAX;
    let mut max_edge = 0.0f64;

    for cell in mesh.polys.iter() {
        match cell.len() {
            3 => n_tri += 1,
            4 => n_quad += 1,
            n if n >= 3 => n_other += 1,
            _ => continue,
        }

        areas.push(polygon_area(mesh, cell));

        for ti in 1..cell.len() - 1 {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[ti] as usize);
            let c = mesh.points.get(cell[ti + 1] as usize);
            let e0 = dist(a, b);
            let e1 = dist(b, c);
            let e2 = dist(c, a);
            aspect_ratios.push(triangle_aspect_ratio(e0, e1, e2, triangle_area(a, b, c)));
        }

        // Edge lengths
        let nc = cell.len();
        for i in 0..nc {
            let pa = mesh.points.get(cell[i] as usize);
            let pb = mesh.points.get(cell[(i + 1) % nc] as usize);
            let a_idx = cell[i] as usize;
            let b_idx = cell[(i + 1) % nc] as usize;
            *edge_set
                .entry((a_idx.min(b_idx), a_idx.max(b_idx)))
                .or_insert(0) += 1;

            let el = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            min_edge = min_edge.min(el);
            max_edge = max_edge.max(el);
        }
    }

    let n_cells = n_tri + n_quad + n_other;
    let n_boundary = edge_set.values().filter(|&&c| c == 1).count();
    let n_non_manifold = edge_set.values().filter(|&&c| c > 2).count();

    let total_area: f64 = areas.iter().sum();
    let mean_area = if !areas.is_empty() {
        total_area / areas.len() as f64
    } else {
        0.0
    };

    let mean_ar = if !aspect_ratios.is_empty() {
        aspect_ratios.iter().sum::<f64>() / aspect_ratios.len() as f64
    } else {
        0.0
    };

    let min_area = if areas.is_empty() {
        0.0
    } else {
        areas.iter().cloned().fold(f64::MAX, f64::min)
    };
    let min_aspect_ratio = if aspect_ratios.is_empty() {
        0.0
    } else {
        aspect_ratios.iter().cloned().fold(f64::MAX, f64::min)
    };

    MeshQualityReport {
        num_points: n_pts,
        num_cells: n_cells,
        num_triangles: n_tri,
        num_quads: n_quad,
        num_other: n_other,
        num_boundary_edges: n_boundary,
        num_non_manifold_edges: n_non_manifold,
        is_closed: n_boundary == 0,
        is_manifold: n_non_manifold == 0,
        is_all_triangles: n_quad == 0 && n_other == 0,
        min_area,
        max_area: areas.iter().cloned().fold(0.0f64, f64::max),
        mean_area,
        min_aspect_ratio,
        max_aspect_ratio: aspect_ratios.iter().cloned().fold(0.0f64, f64::max),
        mean_aspect_ratio: mean_ar,
        min_edge_length: if edge_set.is_empty() { 0.0 } else { min_edge },
        max_edge_length: max_edge,
        surface_area: total_area,
    }
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn triangle_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

fn triangle_aspect_ratio(e0: f64, e1: f64, e2: f64, area: f64) -> f64 {
    if area <= 1e-15 {
        return f64::MAX;
    }
    let longest = e0.max(e1).max(e2);
    let perimeter = e0 + e1 + e2;
    longest * perimeter / (4.0 * 3.0f64.sqrt() * area)
}

fn polygon_area(mesh: &PolyData, cell: &[i64]) -> f64 {
    if cell.len() < 3 {
        return 0.0;
    }

    let a = mesh.points.get(cell[0] as usize);
    let mut total = 0.0;
    for i in 1..cell.len() - 1 {
        let b = mesh.points.get(cell[i] as usize);
        let c = mesh.points.get(cell[i + 1] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        total += 0.5 * (nx * nx + ny * ny + nz * nz).sqrt();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_report() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let report = mesh_quality_report(&mesh);
        assert_eq!(report.num_points, 3);
        assert_eq!(report.num_triangles, 1);
        assert!(report.is_all_triangles);
        assert!(!report.is_closed);
        assert!(report.surface_area > 0.0);
    }

    #[test]
    fn closed_tet() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        let report = mesh_quality_report(&mesh);
        assert!(report.is_closed);
        assert!(report.is_manifold);
    }

    #[test]
    fn display() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let report = mesh_quality_report(&mesh);
        let s = format!("{report}");
        assert!(s.contains("Mesh Quality Report"));
        assert!(s.contains("Points: 3"));
    }

    #[test]
    fn quad_area_uses_full_polygon_fan() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let report = mesh_quality_report(&mesh);
        assert!((report.surface_area - 1.0).abs() < 1e-12);
        assert!((report.mean_area - 1.0).abs() < 1e-12);
    }

    #[test]
    fn triangle_report_uses_vtk_aspect_ratio() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let report = mesh_quality_report(&mesh);
        let expected = (2.0f64.sqrt() * (2.0 + 2.0f64.sqrt())) / (2.0 * 3.0f64.sqrt());
        assert!((report.mean_aspect_ratio - expected).abs() < 1e-12);
        assert!((report.mean_aspect_ratio - 2.0f64.sqrt()).abs() > 1e-3);
    }

    #[test]
    fn empty_report_uses_zero_extrema() {
        let report = mesh_quality_report(&PolyData::new());
        assert_eq!(report.min_area, 0.0);
        assert_eq!(report.min_aspect_ratio, 0.0);
        assert_eq!(report.min_edge_length, 0.0);
    }
}
