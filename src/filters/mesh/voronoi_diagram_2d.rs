use crate::data::{CellArray, Points, PolyData};

/// 2D Voronoi diagram from seed points.
///
/// Builds each bounded Voronoi tile by clipping the bounds rectangle against
/// the perpendicular bisectors to the other seeds, mirroring the core
/// half-space clipping strategy used by VTK's `vtkVoronoi2D`. Polygonal tiles
/// are stored in `polys`; internal tile edges are also emitted as `lines`.
pub fn voronoi_diagram(seeds: &[[f64; 2]], bounds: [[f64; 2]; 2], _resolution: usize) -> PolyData {
    if seeds.is_empty()
        || bounds[0][0] >= bounds[0][1]
        || bounds[1][0] >= bounds[1][1]
        || !bounds.iter().flatten().all(|v| v.is_finite())
    {
        return PolyData::new();
    }

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut out_lines = CellArray::new();

    for (seed_id, seed) in seeds.iter().enumerate() {
        if !seed[0].is_finite() || !seed[1].is_finite() {
            continue;
        }

        let mut tile = bounds_polygon(bounds);
        for (other_id, other) in seeds.iter().enumerate() {
            if seed_id == other_id || !other[0].is_finite() || !other[1].is_finite() {
                continue;
            }
            tile = clip_tile(&tile, *seed, *other);
            if tile.len() < 3 {
                break;
            }
        }

        if tile.len() < 3 {
            continue;
        }

        let base = out_points.len() as i64;
        let mut ids = Vec::with_capacity(tile.len());
        for p in &tile {
            out_points.push([p[0], p[1], 0.0]);
            ids.push(base + ids.len() as i64);
        }
        out_polys.push_cell(&ids);

        for edge in ids.windows(2) {
            let a = tile[(edge[0] - base) as usize];
            let b = tile[(edge[1] - base) as usize];
            if !edge_on_bounds(a, b, bounds) {
                out_lines.push_cell(edge);
            }
        }
        let a = *tile.last().unwrap();
        let b = tile[0];
        if !edge_on_bounds(a, b, bounds) {
            out_lines.push_cell(&[*ids.last().unwrap(), ids[0]]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd
}

fn bounds_polygon(bounds: [[f64; 2]; 2]) -> Vec<[f64; 2]> {
    vec![
        [bounds[0][1], bounds[1][1]],
        [bounds[0][0], bounds[1][1]],
        [bounds[0][0], bounds[1][0]],
        [bounds[0][1], bounds[1][0]],
    ]
}

fn clip_tile(tile: &[[f64; 2]], seed: [f64; 2], other: [f64; 2]) -> Vec<[f64; 2]> {
    if tile.is_empty() {
        return Vec::new();
    }

    let origin = [(seed[0] + other[0]) * 0.5, (seed[1] + other[1]) * 0.5];
    let normal = [other[0] - seed[0], other[1] - seed[1]];
    let mut output = Vec::new();
    let mut previous = *tile.last().unwrap();
    let mut previous_value = evaluate_line(previous, origin, normal);
    let mut previous_inside = previous_value <= 1e-12;

    for &current in tile {
        let current_value = evaluate_line(current, origin, normal);
        let current_inside = current_value <= 1e-12;

        if current_inside != previous_inside {
            output.push(intersect_edge(
                previous,
                current,
                previous_value,
                current_value,
            ));
        }
        if current_inside {
            output.push(current);
        }

        previous = current;
        previous_value = current_value;
        previous_inside = current_inside;
    }

    output
}

fn evaluate_line(x: [f64; 2], origin: [f64; 2], normal: [f64; 2]) -> f64 {
    (x[0] - origin[0]) * normal[0] + (x[1] - origin[1]) * normal[1]
}

fn intersect_edge(a: [f64; 2], b: [f64; 2], value_a: f64, value_b: f64) -> [f64; 2] {
    let denom = value_a - value_b;
    if denom.abs() <= 1e-30 {
        return a;
    }
    let t = value_a / denom;
    [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]
}

fn edge_on_bounds(a: [f64; 2], b: [f64; 2], bounds: [[f64; 2]; 2]) -> bool {
    const EPS: f64 = 1e-10;
    ((a[0] - bounds[0][0]).abs() <= EPS && (b[0] - bounds[0][0]).abs() <= EPS)
        || ((a[0] - bounds[0][1]).abs() <= EPS && (b[0] - bounds[0][1]).abs() <= EPS)
        || ((a[1] - bounds[1][0]).abs() <= EPS && (b[1] - bounds[1][0]).abs() <= EPS)
        || ((a[1] - bounds[1][1]).abs() <= EPS && (b[1] - bounds[1][1]).abs() <= EPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_seed_no_boundaries() {
        let seeds: Vec<[f64; 2]> = vec![[0.5, 0.5]];
        let result = voronoi_diagram(&seeds, [[0.0, 1.0], [0.0, 1.0]], 10);
        // Single seed means all cells belong to same region: no boundaries
        assert_eq!(result.lines.num_cells(), 0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn two_seeds_produces_boundary() {
        let seeds: Vec<[f64; 2]> = vec![[0.25, 0.5], [0.75, 0.5]];
        let result = voronoi_diagram(&seeds, [[0.0, 1.0], [0.0, 1.0]], 20);
        // Should produce boundary lines near x=0.5
        assert!(result.lines.num_cells() > 0);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn empty_seeds_returns_empty() {
        let seeds: Vec<[f64; 2]> = vec![];
        let result = voronoi_diagram(&seeds, [[0.0, 1.0], [0.0, 1.0]], 10);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
