//! Ray-mesh intersection queries.

use crate::data::PolyData;

/// Ray-mesh intersection result.
pub struct RayHit {
    pub point: [f64; 3],
    pub t: f64,
    pub cell_index: usize,
    pub u: f64,
    pub v: f64,
}

/// Cast a ray and find all intersections with the mesh, sorted by distance.
pub fn ray_cast_all(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Vec<RayHit> {
    let Some(dir) = normalize_direction(direction) else {
        return Vec::new();
    };

    let mut hits = Vec::new();
    let poly_offset = mesh.verts.num_cells() + mesh.lines.num_cells();
    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.len() < 3 || !cell_point_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        let mut cell_hit: Option<RayHit> = None;
        for tri in polygon_triangles(cell) {
            update_cell_hit(mesh, origin, dir, tri, poly_offset + ci, &mut cell_hit);
        }
        if let Some(hit) = cell_hit {
            hits.push(hit);
        }
    }
    let strip_offset = poly_offset + mesh.polys.num_cells();
    for (si, strip) in mesh.strips.iter().enumerate() {
        if strip.len() < 3 || !cell_point_ids_are_valid(strip, mesh.points.len()) {
            continue;
        }
        let mut cell_hit: Option<RayHit> = None;
        for tri in strip_triangles(strip) {
            update_cell_hit(mesh, origin, dir, tri, strip_offset + si, &mut cell_hit);
        }
        if let Some(hit) = cell_hit {
            hits.push(hit);
        }
    }
    hits.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
    hits
}

/// Cast a ray and find the first intersection.
pub fn ray_cast_first(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Option<RayHit> {
    let hits = ray_cast_all(mesh, origin, direction);
    hits.into_iter().next()
}

/// Check if a ray intersects the mesh at all.
pub fn ray_intersects(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> bool {
    let Some(dir) = normalize_direction(direction) else {
        return false;
    };

    for cell in mesh.polys.iter() {
        if cell.len() < 3 || !cell_point_ids_are_valid(cell, mesh.points.len()) {
            continue;
        }
        for tri in polygon_triangles(cell) {
            if ray_triangle_by_ids(mesh, origin, dir, tri).is_some() {
                return true;
            }
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 || !cell_point_ids_are_valid(strip, mesh.points.len()) {
            continue;
        }
        for tri in strip_triangles(strip) {
            if ray_triangle_by_ids(mesh, origin, dir, tri).is_some() {
                return true;
            }
        }
    }
    false
}

fn normalize_direction(d: [f64; 3]) -> Option<[f64; 3]> {
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-15 {
        None
    } else {
        Some([d[0] / len, d[1] / len, d[2] / len])
    }
}

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < num_points)
}

fn cell_point_ids_are_valid(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&id| valid_point_id(id, num_points).is_some())
}

fn polygon_triangles(cell: &[i64]) -> impl Iterator<Item = [i64; 3]> + '_ {
    (1..cell.len() - 1).map(move |i| [cell[0], cell[i], cell[i + 1]])
}

fn strip_triangles(strip: &[i64]) -> impl Iterator<Item = [i64; 3]> + '_ {
    strip.windows(3).enumerate().map(|(i, tri)| {
        if i % 2 == 0 {
            [tri[0], tri[1], tri[2]]
        } else {
            [tri[1], tri[0], tri[2]]
        }
    })
}

fn update_cell_hit(
    mesh: &PolyData,
    origin: [f64; 3],
    dir: [f64; 3],
    tri: [i64; 3],
    cell_index: usize,
    cell_hit: &mut Option<RayHit>,
) {
    if let Some((t, u, v)) = ray_triangle_by_ids(mesh, origin, dir, tri) {
        let hit = match cell_hit {
            Some(existing) => t < existing.t,
            None => true,
        };
        if hit {
            *cell_hit = Some(RayHit {
                point: [
                    origin[0] + t * dir[0],
                    origin[1] + t * dir[1],
                    origin[2] + t * dir[2],
                ],
                t,
                cell_index,
                u,
                v,
            });
        }
    }
}

fn ray_triangle_by_ids(
    mesh: &PolyData,
    origin: [f64; 3],
    dir: [f64; 3],
    tri: [i64; 3],
) -> Option<(f64, f64, f64)> {
    let a = mesh.points.get(tri[0] as usize);
    let b = mesh.points.get(tri[1] as usize);
    let c = mesh.points.get(tri[2] as usize);
    ray_triangle(origin, dir, a, b, c)
}

fn ray_triangle(
    o: [f64; 3],
    d: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> Option<(f64, f64, f64)> {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let h = cross(d, e2);
    let det = dot(e1, h);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let s = [o[0] - a[0], o[1] - a[1], o[2] - a[2]];
    let u = inv * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return None;
    }
    let q = cross(s, e1);
    let v = inv * dot(d, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = inv * dot(e2, q);
    if t > 1e-12 {
        Some((t, u, v))
    } else {
        None
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hit() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let hit = ray_cast_first(&mesh, [1.0, 1.0, 5.0], [0.0, 0.0, -1.0]);
        assert!(hit.is_some());
        let h = hit.unwrap();
        assert!((h.point[2] - 0.0).abs() < 1e-10);
        assert!((h.t - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_miss() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(!ray_intersects(&mesh, [10.0, 10.0, 5.0], [0.0, 0.0, -1.0]));
    }
    #[test]
    fn test_all() {
        // Two parallel triangles
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [0.0, 0.0, 3.0],
                [2.0, 0.0, 3.0],
                [1.0, 2.0, 3.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let hits = ray_cast_all(&mesh, [1.0, 1.0, 10.0], [0.0, 0.0, -1.0]);
        assert_eq!(hits.len(), 2);
        assert!(hits[0].t < hits[1].t);
    }

    #[test]
    fn test_non_unit_direction_reports_distance() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let hit = ray_cast_first(&mesh, [1.0, 1.0, 5.0], [0.0, 0.0, -2.0]).unwrap();
        assert!((hit.t - 5.0).abs() < 1e-10);
    }

    #[test]
    fn polygon_fan_internal_edge_counts_as_one_cell_hit() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let hits = ray_cast_all(&mesh, [0.5, 0.5, 1.0], [0.0, 0.0, -1.0]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].cell_index, 0);
    }

    #[test]
    fn invalid_cell_ids_are_skipped() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.polys.push_cell(&[-1, 0, 1]);
        mesh.polys.push_cell(&[0, 1, 99]);

        let hits = ray_cast_all(&mesh, [1.0, 1.0, 5.0], [0.0, 0.0, -1.0]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].cell_index, 0);
        assert!(ray_intersects(&mesh, [1.0, 1.0, 5.0], [0.0, 0.0, -1.0]));
    }

    #[test]
    fn triangle_strips_are_intersected() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.strips.push_cell(&[0, 1, 2, 3]);

        let hits = ray_cast_all(&mesh, [0.75, 0.75, 1.0], [0.0, 0.0, -1.0]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].cell_index, 0);
        assert!(ray_intersects(&mesh, [0.75, 0.75, 1.0], [0.0, 0.0, -1.0]));
    }

    #[test]
    fn cell_index_uses_polydata_cell_order() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.strips.push_cell(&[1, 3, 2]);

        let poly_hit = ray_cast_first(&mesh, [0.25, 0.25, 1.0], [0.0, 0.0, -1.0]).unwrap();
        assert_eq!(poly_hit.cell_index, 2);

        let strip_hit = ray_cast_first(&mesh, [0.75, 0.75, 1.0], [0.0, 0.0, -1.0]).unwrap();
        assert_eq!(strip_hit.cell_index, 3);
    }
}
