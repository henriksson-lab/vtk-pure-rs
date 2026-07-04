//! Compute mesh boundary perimeter length.
use crate::data::PolyData;
pub fn boundary_perimeter(mesh: &PolyData) -> f64 {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        add_polygon_edges(cell, mesh.points.len(), &mut ec);
    }
    for strip in mesh.strips.iter() {
        add_triangle_strip_edges(strip, mesh.points.len(), &mut ec);
    }
    ec.iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&(a, b), _)| {
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
        })
        .sum()
}
pub fn total_edge_length(mesh: &PolyData) -> f64 {
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut total = 0.0;
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            accumulate_edge(mesh, cell[i], cell[(i + 1) % nc], &mut seen, &mut total);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            accumulate_edge(mesh, edge[0], edge[1], &mut seen, &mut total);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            accumulate_edge(mesh, tri[0], tri[1], &mut seen, &mut total);
            accumulate_edge(mesh, tri[1], tri[2], &mut seen, &mut total);
            accumulate_edge(mesh, tri[2], tri[0], &mut seen, &mut total);
        }
    }
    total
}

fn add_polygon_edges(
    cell: &[i64],
    npoints: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    let nc = cell.len();
    if nc < 2 {
        return;
    }
    for i in 0..nc {
        add_counted_edge(cell[i], cell[(i + 1) % nc], npoints, edge_count);
    }
}

fn add_triangle_strip_edges(
    strip: &[i64],
    npoints: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    if strip.len() < 3 {
        return;
    }
    for i in 0..strip.len() - 2 {
        let tri = if i % 2 == 0 {
            [strip[i], strip[i + 1], strip[i + 2]]
        } else {
            [strip[i + 1], strip[i], strip[i + 2]]
        };
        add_counted_edge(tri[0], tri[1], npoints, edge_count);
        add_counted_edge(tri[1], tri[2], npoints, edge_count);
        add_counted_edge(tri[2], tri[0], npoints, edge_count);
    }
}

fn add_counted_edge(
    a_id: i64,
    b_id: i64,
    npoints: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    let Some(a) = valid_point_id(a_id, npoints) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, npoints) else {
        return;
    };
    if a == b {
        return;
    }
    *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
}

fn accumulate_edge(
    mesh: &PolyData,
    a_id: i64,
    b_id: i64,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    total: &mut f64,
) {
    let Some(a) = valid_point_id(a_id, mesh.points.len()) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, mesh.points.len()) else {
        return;
    };
    if a == b || !seen.insert((a.min(b), a.max(b))) {
        return;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    *total += ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
}

fn valid_point_id(point_id: i64, npoints: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&id| id < npoints)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_perimeter() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let p = boundary_perimeter(&m);
        assert!(p > 2.0);
    }
    #[test]
    fn test_total() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let t = total_edge_length(&m);
        assert!((t - (1.0 + 1.0 + 2.0f64.sqrt())).abs() < 1e-10);
    }

    #[test]
    fn strips_are_counted_as_triangles() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.points.push([1.0, 1.0, 0.0]);
        m.strips.push_cell(&[0, 1, 2, 3]);

        assert!((boundary_perimeter(&m) - 4.0).abs() < 1e-10);
        assert!((total_edge_length(&m) - (4.0 + 2.0f64.sqrt())).abs() < 1e-10);
    }
}
