use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Rotational extrusion of polygonal data around the Z axis.
///
/// Takes a PolyData whose line cells represent a profile and sweeps it around
/// the Z axis by `angle` degrees with `resolution` steps. This mirrors VTK's
/// default `vtkRotationalExtrusionFilter` path for line and vertex cells:
/// vertices generate lines, and lines generate triangle strips.
///
/// The profile should be in a half-plane away from the rotation axis for
/// non-degenerate geometry.
pub fn rotation_extrude(input: &PolyData, angle: f64, resolution: usize) -> PolyData {
    let resolution = resolution.max(1);
    let num_pts = input.points.len();
    let angle_incr = angle.to_radians() / resolution as f64;

    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();

    for i in 0..=resolution {
        let theta = i as f64 * angle_incr;
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        for pt_id in 0..num_pts {
            let p = input.points.get(pt_id);
            points.push([
                p[0] * cos_t - p[1] * sin_t,
                p[0] * sin_t + p[1] * cos_t,
                p[2],
            ]);
        }
    }

    for cell in input.verts.iter() {
        for &pt_id in cell.iter() {
            let mut line = Vec::with_capacity(resolution + 1);
            for j in 0..=resolution {
                line.push(pt_id + (j * num_pts) as i64);
            }
            lines.push_cell(&line);
        }
    }

    if angle != 360.0 {
        for cell in input.polys.iter() {
            polys.push_cell(cell);
            let cap: Vec<i64> = cell
                .iter()
                .map(|&pt_id| pt_id + (resolution * num_pts) as i64)
                .collect();
            polys.push_cell(&cap);
        }

        for cell in input.strips.iter() {
            strips.push_cell(cell);
            let cap: Vec<i64> = cell
                .iter()
                .map(|&pt_id| pt_id + (resolution * num_pts) as i64)
                .collect();
            strips.push_cell(&cap);
        }
    }

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }

        for edge in cell.windows(2) {
            let p1 = edge[0];
            let p2 = edge[1];
            let mut strip = Vec::with_capacity(2 * (resolution + 1));
            for j in 0..=resolution {
                let offset = (j * num_pts) as i64;
                strip.push(p2 + offset);
                strip.push(p1 + offset);
            }
            strips.push_cell(&strip);
        }
    }

    for (p1, p2) in boundary_edges(input) {
        let mut strip = Vec::with_capacity(2 * (resolution + 1));
        for j in 0..=resolution {
            let offset = (j * num_pts) as i64;
            strip.push(p2 + offset);
            strip.push(p1 + offset);
        }
        strips.push_cell(&strip);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd.polys = polys;
    pd.strips = strips;
    pd
}

fn boundary_edges(input: &PolyData) -> Vec<(i64, i64)> {
    let mut edge_counts: HashMap<(i64, i64), usize> = HashMap::new();
    let mut ordered_edges = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            add_edge(
                cell[i],
                cell[(i + 1) % cell.len()],
                &mut edge_counts,
                &mut ordered_edges,
            );
        }
    }

    for cell in input.strips.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 0..cell.len() - 2 {
            add_edge(cell[i], cell[i + 1], &mut edge_counts, &mut ordered_edges);
            add_edge(
                cell[i + 1],
                cell[i + 2],
                &mut edge_counts,
                &mut ordered_edges,
            );
            add_edge(cell[i + 2], cell[i], &mut edge_counts, &mut ordered_edges);
        }
    }

    ordered_edges
        .into_iter()
        .filter_map(|(p1, p2, key)| (edge_counts[&key] == 1).then_some((p1, p2)))
        .collect()
}

fn add_edge(
    p1: i64,
    p2: i64,
    edge_counts: &mut HashMap<(i64, i64), usize>,
    ordered_edges: &mut Vec<(i64, i64, (i64, i64))>,
) {
    let key = if p1 < p2 { (p1, p2) } else { (p2, p1) };
    edge_counts
        .entry(key)
        .and_modify(|count| *count += 1)
        .or_insert(1);
    ordered_edges.push((p1, p2, key));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revolve_line_segment() {
        // A vertical line at x=1 -> should produce a cylinder strip
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = rotation_extrude(&pd, 360.0, 8);
        // VTK keeps the base layer plus resolution generated layers.
        assert_eq!(result.points.len(), 18);
        assert_eq!(result.strips.num_cells(), 1);
        assert_eq!(result.strips.cell(0).len(), 18);
    }

    #[test]
    fn revolve_partial() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = rotation_extrude(&pd, 180.0, 6);
        assert_eq!(result.points.len(), 14);
        assert_eq!(result.strips.num_cells(), 1);
        assert_eq!(result.strips.cell(0).len(), 14);
        let last = result.points.get(12);
        assert!((last[0] + 1.0).abs() < 1e-10);
        assert!(last[1].abs() < 1e-10);
    }

    #[test]
    fn revolve_profile() {
        // L-shaped profile: two input line segments produce two strips.
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 1.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = rotation_extrude(&pd, 360.0, 4);
        assert_eq!(result.points.len(), 15);
        assert_eq!(result.strips.num_cells(), 2);
        assert_eq!(result.strips.cell(0), &[1, 0, 4, 3, 7, 6, 10, 9, 13, 12]);
    }

    #[test]
    fn vertices_generate_lines() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.verts.push_cell(&[0]);

        let result = rotation_extrude(&pd, 90.0, 3);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.lines.cell(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = rotation_extrude(&pd, 360.0, 8);
        assert_eq!(result.strips.num_cells(), 0);
    }

    #[test]
    fn polygon_boundary_edges_generate_strips() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );

        let result = rotation_extrude(&pd, 360.0, 4);

        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.strips.num_cells(), 3);
        assert_eq!(result.strips.cell(0), &[1, 0, 4, 3, 7, 6, 10, 9, 13, 12]);
    }

    #[test]
    fn partial_polygon_sweep_is_capped() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 0.0, 1.0]],
            vec![[0, 1, 2]],
        );

        let result = rotation_extrude(&pd, 180.0, 2);

        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[6, 7, 8]);
    }
}
