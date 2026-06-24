use crate::data::{AnyDataArray, DataArray, PolyData};

/// Determine which points are inside a closed triangular surface.
///
/// Uses a ray-casting approach: for each probe point, counts how many
/// times a ray in the +X direction intersects the surface triangles.
/// An odd count means the point is inside.
///
/// Adds a "InsideOut" scalar array (1.0 = inside, 0.0 = outside) to
/// the probe's point data.
pub fn select_enclosed_points(surface: &PolyData, probe: &PolyData) -> PolyData {
    let mut pd = probe.clone();
    let n = probe.points.len();

    // Collect triangles
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3])> = surface
        .polys
        .iter()
        .flat_map(|cell| {
            let p0 = surface.points.get(cell[0] as usize);
            (1..cell.len() - 1).map(move |i| {
                (
                    p0,
                    surface.points.get(cell[i] as usize),
                    surface.points.get(cell[i + 1] as usize),
                )
            })
        })
        .collect();

    let mut inside = vec![0.0f64; n];

    for (pi, val) in inside.iter_mut().enumerate() {
        let p = probe.points.get(pi);
        let mut count = 0;

        for &(v0, v1, v2) in &tris {
            if ray_intersects_triangle(p, v0, v1, v2) {
                count += 1;
            }
        }

        if count % 2 == 1 {
            *val = 1.0;
        }
    }

    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "InsideOut",
            inside,
            1,
        )));
    pd
}

/// Möller–Trumbore ray-triangle intersection.
/// Ray origin = p, direction = +X.
fn ray_intersects_triangle(origin: [f64; 3], v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> bool {
    let dir = [1.0, 0.0, 0.0]; // +X ray
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let h = cross(dir, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-12 {
        return false;
    }

    let f = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u = f * dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return false;
    }

    let q = cross(s, e1);
    let v = f * dot(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }

    let t = f * dot(e2, q);
    t > 1e-12 // Intersection in +X direction
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
    fn point_inside_tetra() {
        // Hand-built tetrahedron with outward-facing normals
        let surface = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 2.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );

        let mut probe = PolyData::new();
        probe.points.push([1.0, 0.5, 0.5]); // inside the tetrahedron
        probe.points.push([10.0, 10.0, 10.0]); // far outside

        let result = select_enclosed_points(&surface, &probe);
        let arr = result.point_data().get_array("InsideOut").unwrap();
        assert_eq!(arr.num_tuples(), 2);
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-10, "inside point got {}", val[0]);
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0]).abs() < 1e-10, "outside point got {}", val[0]);
    }

    #[test]
    fn all_outside() {
        let surface = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut probe = PolyData::new();
        probe.points.push([10.0, 10.0, 10.0]);

        let result = select_enclosed_points(&surface, &probe);
        let arr = result.point_data().get_array("InsideOut").unwrap();
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0]).abs() < 1e-10);
    }
}
