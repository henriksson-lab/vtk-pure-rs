//! Extract points enclosed by a closed surface mesh.
//!
//! Uses ray-casting to test if points lie inside or outside a closed
//! triangle mesh.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Test which points lie inside a closed surface mesh.
///
/// Returns a vector of booleans, one per point in `probe`.
pub fn points_inside_surface(probe: &PolyData, surface: &PolyData) -> Vec<bool> {
    let n = probe.points.len();
    let tris = collect_triangles(surface);

    (0..n)
        .map(|i| {
            let p = probe.points.get(i);
            is_inside_ray_cast(p, &tris)
        })
        .collect()
}

/// Extract only points that lie inside the surface.
pub fn extract_enclosed_points(probe: &PolyData, surface: &PolyData) -> PolyData {
    let inside = points_inside_surface(probe, surface);
    let mut points = Points::<f64>::new();
    for (i, &is_in) in inside.iter().enumerate() {
        if is_in {
            points.push(probe.points.get(i));
        }
    }
    let mut result = PolyData::new();
    result.points = points;
    result
}

/// Mark points with an "InsideSurface" array (1=inside, 0=outside).
pub fn mark_enclosed_points(probe: &PolyData, surface: &PolyData) -> PolyData {
    let inside = points_inside_surface(probe, surface);
    let data: Vec<f64> = inside.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();

    let mut result = probe.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "InsideSurface",
            data,
            1,
        )));
    result
}

type Triangle = [[f64; 3]; 3];

fn collect_triangles(mesh: &PolyData) -> Vec<Triangle> {
    let mut tris = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() >= 3 {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            tris.push([a, b, c]);
            // Triangulate polygons with more than 3 vertices
            for i in 2..cell.len() - 1 {
                let bi = mesh.points.get(cell[i] as usize);
                let ci = mesh.points.get(cell[i + 1] as usize);
                tris.push([a, bi, ci]);
            }
        }
    }
    tris
}

fn is_inside_ray_cast(point: [f64; 3], triangles: &[Triangle]) -> bool {
    // Cast ray in +X direction and count intersections
    let mut count = 0;
    for tri in triangles {
        if ray_triangle_intersect(point, [1.0, 0.0, 0.0], tri) {
            count += 1;
        }
    }
    count % 2 == 1
}

fn ray_triangle_intersect(origin: [f64; 3], dir: [f64; 3], tri: &Triangle) -> bool {
    let e1 = sub(tri[1], tri[0]);
    let e2 = sub(tri[2], tri[0]);
    let h = cross(dir, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-12 {
        return false;
    }
    let f = 1.0 / a;
    let s = sub(origin, tri[0]);
    let u = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return false;
    }
    let q = cross(s, e1);
    let v = f * dot(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = f * dot(e2, q);
    t > 1e-8 // intersection ahead of ray
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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

    fn make_closed_tetrahedron() -> PolyData {
        // A closed tetrahedron centered roughly at (0.5, 0.5, 0.5)
        let pts = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
            [1.0, 0.7, 2.0],
        ];
        let tris = vec![
            [0, 2, 1], // bottom (outward-facing)
            [0, 1, 3], // front
            [1, 2, 3], // right
            [2, 0, 3], // left
        ];
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn point_inside_tetrahedron() {
        let tet = make_closed_tetrahedron();
        let mut probe = PolyData::new();
        probe.points = Points::from(vec![
            [1.0, 0.5, 0.3], // inside (centroid-ish)
            [5.0, 5.0, 5.0], // clearly outside
        ]);
        let inside = points_inside_surface(&probe, &tet);
        assert!(inside[0], "interior point should be inside");
        assert!(!inside[1], "far point should be outside");
    }

    #[test]
    fn extract_inside() {
        let tet = make_closed_tetrahedron();
        let mut probe = PolyData::new();
        probe.points = Points::from(vec![
            [1.0, 0.5, 0.3], // inside
            [5.0, 5.0, 5.0], // outside
            [0.8, 0.4, 0.2], // inside
        ]);
        let result = extract_enclosed_points(&probe, &tet);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn mark_enclosed() {
        let tet = make_closed_tetrahedron();
        let mut probe = PolyData::new();
        probe.points = Points::from(vec![[1.0, 0.5, 0.3], [5.0, 5.0, 5.0]]);
        let result = mark_enclosed_points(&probe, &tet);
        let arr = result.point_data().get_array("InsideSurface").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
