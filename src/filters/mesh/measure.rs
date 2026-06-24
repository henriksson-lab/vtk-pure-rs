use crate::data::PolyData;

/// Compute mesh statistics: point count, cell count, area, edge stats.
#[derive(Debug, Clone)]
pub struct MeshMeasurements {
    pub num_points: usize,
    pub num_triangles: usize,
    pub num_quads: usize,
    pub num_other_polys: usize,
    pub num_lines: usize,
    pub total_area: f64,
    pub total_edge_length: f64,
    pub min_edge_length: f64,
    pub max_edge_length: f64,
    pub avg_edge_length: f64,
}

/// Measure a PolyData mesh comprehensively.
pub fn measure_mesh(input: &PolyData) -> MeshMeasurements {
    let mut num_tris = 0usize;
    let mut num_quads = 0usize;
    let mut num_other = 0usize;
    let mut total_area = 0.0;
    let mut min_edge = f64::MAX;
    let mut max_edge = 0.0f64;
    let mut sum_edge = 0.0;
    let mut edge_count = 0usize;

    for cell in input.polys.iter() {
        match cell.len() {
            3 => num_tris += 1,
            4 => num_quads += 1,
            n if n > 4 => num_other += 1,
            _ => {}
        }

        if cell.len() >= 3 {
            let v0 = input.points.get(cell[0] as usize);
            for i in 1..cell.len() - 1 {
                let v1 = input.points.get(cell[i] as usize);
                let v2 = input.points.get(cell[i + 1] as usize);
                total_area += tri_area(v0, v1, v2);
            }
            for i in 0..cell.len() {
                let a = input.points.get(cell[i] as usize);
                let b = input.points.get(cell[(i + 1) % cell.len()] as usize);
                let d = dist(a, b);
                min_edge = min_edge.min(d);
                max_edge = max_edge.max(d);
                sum_edge += d;
                edge_count += 1;
            }
        }
    }

    let mut total_line_len = 0.0;
    let mut num_lines = 0;
    for cell in input.lines.iter() {
        num_lines += 1;
        for i in 0..cell.len() - 1 {
            total_line_len += dist(
                input.points.get(cell[i] as usize),
                input.points.get(cell[i + 1] as usize),
            );
        }
    }

    if min_edge == f64::MAX {
        min_edge = 0.0;
    }

    MeshMeasurements {
        num_points: input.points.len(),
        num_triangles: num_tris,
        num_quads: num_quads,
        num_other_polys: num_other,
        num_lines: num_lines,
        total_area: total_area,
        total_edge_length: sum_edge + total_line_len,
        min_edge_length: min_edge,
        max_edge_length: max_edge,
        avg_edge_length: if edge_count > 0 {
            sum_edge / edge_count as f64
        } else {
            0.0
        },
    }
}

fn tri_area(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> f64 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let c = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    0.5 * (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt()
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let m = measure_mesh(&pd);
        assert_eq!(m.num_points, 3);
        assert_eq!(m.num_triangles, 1);
        assert!((m.total_area - 0.5).abs() < 1e-10);
        assert!(m.min_edge_length > 0.0);
    }

    #[test]
    fn measure_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let m = measure_mesh(&pd);
        assert_eq!(m.num_quads, 1);
        assert!((m.total_area - 1.0).abs() < 1e-10);
    }

    #[test]
    fn measure_with_lines() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let m = measure_mesh(&pd);
        assert_eq!(m.num_lines, 1);
        assert!((m.total_edge_length - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let m = measure_mesh(&pd);
        assert_eq!(m.num_points, 0);
        assert_eq!(m.total_area, 0.0);
    }
}
