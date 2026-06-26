use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Compute a 2D Voronoi diagram from a set of points.
///
/// Constructs each Voronoi tile by clipping a padded bounding rectangle with
/// the perpendicular bisector half-spaces induced by the other input points.
/// Returns a PolyData with polygon cells and a "SiteId" cell
/// data array mapping each cell to its generator point.
///
/// Points are projected to the XY plane. The result is clipped to a
/// bounding rectangle padded by `padding` times the point-set diagonal.
pub fn voronoi_2d(input: &PolyData, padding: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            [p[0], p[1]]
        })
        .collect();

    let (min_x, max_x, min_y, max_y) = bounds_2d(&pts);
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let diagonal = (dx * dx + dy * dy).sqrt();
    let pad = padding.max(0.0) * diagonal.max(1.0);
    let bounds = [min_x - pad, max_x + pad, min_y - pad, max_y + pad];

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut site_ids: Vec<f64> = Vec::new();

    for pi in 0..n {
        let site = pts[pi];
        let mut tile = vec![
            [bounds[0], bounds[2]],
            [bounds[1], bounds[2]],
            [bounds[1], bounds[3]],
            [bounds[0], bounds[3]],
        ];

        for (pj, &other) in pts.iter().enumerate() {
            if pi == pj || same_point(site, other) {
                continue;
            }
            tile = clip_to_nearest_halfspace(&tile, site, other);
            if tile.len() < 3 {
                break;
            }
        }

        if tile.len() >= 3 {
            let base = out_points.len() as i64;
            let mut cell_ids = Vec::with_capacity(tile.len());
            for (i, p) in tile.iter().enumerate() {
                out_points.push([p[0], p[1], 0.0]);
                cell_ids.push(base + i as i64);
            }
            out_polys.push_cell(&cell_ids);
            site_ids.push(pi as f64);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SiteId", site_ids, 1,
        )));
    pd
}

fn bounds_2d(pts: &[[f64; 2]]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for p in pts {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }
    if min_x == max_x {
        min_x -= 0.5;
        max_x += 0.5;
    }
    if min_y == max_y {
        min_y -= 0.5;
        max_y += 0.5;
    }
    (min_x, max_x, min_y, max_y)
}

fn clip_to_nearest_halfspace(poly: &[[f64; 2]], site: [f64; 2], other: [f64; 2]) -> Vec<[f64; 2]> {
    let mut out = Vec::new();
    let mut prev = poly[poly.len() - 1];
    let mut prev_val = bisector_value(prev, site, other);
    let mut prev_inside = prev_val <= 1e-12;

    for &curr in poly {
        let curr_val = bisector_value(curr, site, other);
        let curr_inside = curr_val <= 1e-12;

        if curr_inside != prev_inside {
            let t = prev_val / (prev_val - curr_val);
            out.push([
                prev[0] + t * (curr[0] - prev[0]),
                prev[1] + t * (curr[1] - prev[1]),
            ]);
        }
        if curr_inside {
            out.push(curr);
        }

        prev = curr;
        prev_val = curr_val;
        prev_inside = curr_inside;
    }

    out
}

fn bisector_value(x: [f64; 2], site: [f64; 2], other: [f64; 2]) -> f64 {
    2.0 * (x[0] * (other[0] - site[0]) + x[1] * (other[1] - site[1]))
        + site[0] * site[0]
        + site[1] * site[1]
        - other[0] * other[0]
        - other[1] * other[1]
}

fn same_point(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() <= 1e-15 && (a[1] - b[1]).abs() <= 1e-15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_voronoi() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        let result = voronoi_2d(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 9);
        assert!(result.cell_data().get_array("SiteId").is_some());
    }

    #[test]
    fn too_few_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let result = voronoi_2d(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn hexagonal_layout() {
        let mut pd = PolyData::new();
        // 7 points: center + 6 surrounding
        pd.points.push([0.0, 0.0, 0.0]);
        for i in 0..6 {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([angle.cos(), angle.sin(), 0.0]);
        }
        let result = voronoi_2d(&pd, 1.0);
        assert!(result.polys.num_cells() >= 1);
    }
}
