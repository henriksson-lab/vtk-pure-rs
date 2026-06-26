use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute mesh quality statistics.
///
/// Returns a struct with min/max/mean values for edge length,
/// triangle area, and aspect ratio.
#[derive(Debug, Clone)]
pub struct MeshQualityStats {
    pub min_edge_length: f64,
    pub max_edge_length: f64,
    pub mean_edge_length: f64,
    pub min_area: f64,
    pub max_area: f64,
    pub mean_area: f64,
    pub min_aspect_ratio: f64,
    pub max_aspect_ratio: f64,
    pub mean_aspect_ratio: f64,
    pub num_triangles: usize,
    pub num_degenerate: usize,
}

/// Compute mesh quality statistics for a triangle mesh.
pub fn mesh_quality(input: &PolyData) -> MeshQualityStats {
    let mut min_edge = f64::MAX;
    let mut max_edge = 0.0f64;
    let mut sum_edge = 0.0;
    let mut edge_count = 0usize;

    let mut min_area = f64::MAX;
    let mut max_area = 0.0f64;
    let mut sum_area = 0.0;

    let mut min_ar = f64::MAX;
    let mut max_ar = 0.0f64;
    let mut sum_ar = 0.0;

    let mut num_tris = 0usize;
    let mut num_degen = 0usize;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let v0 = input.points.get(cell[0] as usize);
        for ti in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[ti] as usize);
            let v2 = input.points.get(cell[ti + 1] as usize);

            let e0 = dist(v0, v1);
            let e1 = dist(v1, v2);
            let e2 = dist(v2, v0);

            min_edge = min_edge.min(e0).min(e1).min(e2);
            max_edge = max_edge.max(e0).max(e1).max(e2);
            sum_edge += e0 + e1 + e2;
            edge_count += 3;

            let area = triangle_area(v0, v1, v2);
            min_area = min_area.min(area);
            max_area = max_area.max(area);
            sum_area += area;

            let ar = triangle_aspect_ratio(e0, e1, e2, area);

            if area < 1e-15 {
                num_degen += 1;
            }

            if ar < f64::MAX {
                min_ar = min_ar.min(ar);
                max_ar = max_ar.max(ar);
                sum_ar += ar;
            }

            num_tris += 1;
        }
    }

    if num_tris == 0 {
        return MeshQualityStats {
            min_edge_length: 0.0,
            max_edge_length: 0.0,
            mean_edge_length: 0.0,
            min_area: 0.0,
            max_area: 0.0,
            mean_area: 0.0,
            min_aspect_ratio: 0.0,
            max_aspect_ratio: 0.0,
            mean_aspect_ratio: 0.0,
            num_triangles: 0,
            num_degenerate: 0,
        };
    }

    MeshQualityStats {
        min_edge_length: min_edge,
        max_edge_length: max_edge,
        mean_edge_length: sum_edge / edge_count as f64,
        min_area: min_area,
        max_area: max_area,
        mean_area: sum_area / num_tris as f64,
        min_aspect_ratio: min_ar,
        max_aspect_ratio: max_ar,
        mean_aspect_ratio: sum_ar / num_tris as f64,
        num_triangles: num_tris,
        num_degenerate: num_degen,
    }
}

/// Add per-cell quality metrics as cell data arrays.
///
/// Adds "AspectRatio" and "CellArea" cell data arrays.
pub fn mesh_quality_arrays(input: &PolyData) -> PolyData {
    let mut aspect_ratios = Vec::new();
    let mut areas = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            aspect_ratios.push(0.0);
            areas.push(0.0);
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);

        let e0 = dist(v0, v1);
        let e1 = dist(v1, v2);
        let e2 = dist(v2, v0);
        let area = triangle_area(v0, v1, v2);
        let ar = triangle_aspect_ratio(e0, e1, e2, area);

        aspect_ratios.push(if ar < f64::MAX { ar } else { 0.0 });
        areas.push(area);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AspectRatio",
            aspect_ratios,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("CellArea", areas, 1)));
    pd
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn triangle_area(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> f64 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equilateral_quality() {
        let mut pd = PolyData::new();
        let h = (3.0f64).sqrt() / 2.0;
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let stats = mesh_quality(&pd);
        assert_eq!(stats.num_triangles, 1);
        assert_eq!(stats.num_degenerate, 0);
        assert!((stats.min_aspect_ratio - 1.0).abs() < 1e-10); // equilateral = 1:1
    }

    #[test]
    fn quality_arrays() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = mesh_quality_arrays(&pd);
        assert!(result.cell_data().get_array("AspectRatio").is_some());
        assert!(result.cell_data().get_array("CellArea").is_some());
    }

    #[test]
    fn vtk_triangle_aspect_ratio_not_edge_ratio() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let stats = mesh_quality(&pd);
        let expected = (2.0f64.sqrt() * (2.0 + 2.0f64.sqrt())) / (2.0 * 3.0f64.sqrt());
        assert!((stats.mean_aspect_ratio - expected).abs() < 1e-12);
        assert!((stats.mean_aspect_ratio - 2.0f64.sqrt()).abs() > 1e-3);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let stats = mesh_quality(&pd);
        assert_eq!(stats.num_triangles, 0);
    }
}
