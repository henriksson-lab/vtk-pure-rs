use crate::data::{AnyDataArray, DataArray, DataSet, ImageData, PolyData};

/// Convert a PolyData surface to an ImageData by sampling a signed distance field.
///
/// Creates an ImageData grid with the given dimensions that spans the bounding
/// box of the input mesh (with padding). Each voxel stores the approximate
/// distance to the nearest surface point. Interior voxels (determined by
/// ray-casting parity) are negative.
///
/// The output has a "Distance" point data array.
pub fn poly_data_to_image_data(input: &PolyData, dimensions: [usize; 3]) -> ImageData {
    let bb = input.bounds();
    let padding = 0.1 * bb.diagonal_length().max(1e-10);

    let origin = [bb.x_min - padding, bb.y_min - padding, bb.z_min - padding];
    let spacing = [
        (bb.x_max - bb.x_min + 2.0 * padding) / (dimensions[0] as f64 - 1.0).max(1.0),
        (bb.y_max - bb.y_min + 2.0 * padding) / (dimensions[1] as f64 - 1.0).max(1.0),
        (bb.z_max - bb.z_min + 2.0 * padding) / (dimensions[2] as f64 - 1.0).max(1.0),
    ];

    let mut img = ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
    img.set_origin(origin);
    img.set_spacing(spacing);

    // Collect triangles
    let tris: Vec<[[f64; 3]; 3]> = input
        .polys
        .iter()
        .flat_map(|cell| {
            let v0 = input.points.get(cell[0] as usize);
            (1..cell.len() - 1).map(move |i| {
                [
                    v0,
                    input.points.get(cell[i] as usize),
                    input.points.get(cell[i + 1] as usize),
                ]
            })
        })
        .collect();

    let n_pts = dimensions[0] * dimensions[1] * dimensions[2];
    let mut distances = vec![0.0f64; n_pts];

    for k in 0..dimensions[2] {
        for j in 0..dimensions[1] {
            for i in 0..dimensions[0] {
                let p = [
                    origin[0] + i as f64 * spacing[0],
                    origin[1] + j as f64 * spacing[1],
                    origin[2] + k as f64 * spacing[2],
                ];

                // Find minimum distance to any triangle
                let mut min_d2 = f64::MAX;
                for tri in &tris {
                    let cp = closest_point_on_triangle(p, tri);
                    let d2 = dist2(p, cp);
                    if d2 < min_d2 {
                        min_d2 = d2;
                    }
                }

                let mut dist = min_d2.sqrt();

                // Determine sign by ray-casting (+X direction)
                let mut crossings = 0u32;
                for tri in &tris {
                    if ray_hits_triangle(p, tri) {
                        crossings += 1;
                    }
                }
                if crossings % 2 == 1 {
                    dist = -dist; // inside
                }

                let idx = k * dimensions[1] * dimensions[0] + j * dimensions[0] + i;
                distances[idx] = dist;
            }
        }
    }

    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    img.point_data_mut().set_active_scalars("Distance");
    img
}

fn closest_point_on_triangle(p: [f64; 3], tri: &[[f64; 3]; 3]) -> [f64; 3] {
    let a = tri[0];
    let b = tri[1];
    let c = tri[2];
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return add(a, scale(ab, v));
    }

    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return add(a, scale(ac, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add(b, scale(sub(c, b), w));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    add(a, add(scale(ab, v), scale(ac, w)))
}

fn ray_hits_triangle(origin: [f64; 3], tri: &[[f64; 3]; 3]) -> bool {
    let e1 = sub(tri[1], tri[0]);
    let e2 = sub(tri[2], tri[0]);
    let dir = [1.0, 0.0, 0.0];
    let h = cross(dir, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-12 {
        return false;
    }
    let f = 1.0 / a;
    let s = sub(origin, tri[0]);
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
    t > 1e-12
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box_mesh() -> PolyData {
        let mut pd = PolyData::new();
        let corners = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        for c in &corners {
            pd.points.push(*c);
        }
        let faces = [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [2, 3, 7, 6],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
        ];
        for f in &faces {
            pd.polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
            pd.polys.push_cell(&[f[0] as i64, f[2] as i64, f[3] as i64]);
        }
        pd
    }

    #[test]
    fn creates_image_data() {
        let pd = make_box_mesh();
        let img = poly_data_to_image_data(&pd, [5, 5, 5]);
        assert_eq!(img.dimensions(), [5, 5, 5]);
        assert!(img.point_data().get_array("Distance").is_some());
    }

    #[test]
    fn has_varying_distances() {
        let pd = make_box_mesh();
        let img = poly_data_to_image_data(&pd, [8, 8, 8]);
        let arr = img.point_data().get_array("Distance").unwrap();

        // Distance field should have different values (not all the same)
        let mut buf = [0.0f64];
        let mut min_d = f64::MAX;
        let mut max_d = f64::MIN;
        for i in 0..8 * 8 * 8 {
            arr.tuple_as_f64(i, &mut buf);
            min_d = min_d.min(buf[0]);
            max_d = max_d.max(buf[0]);
        }
        assert!(max_d > min_d, "distance field should have variation");
        assert!(
            min_d >= 0.0 || max_d > 0.0,
            "should have some positive distances"
        );
    }

    #[test]
    fn exterior_positive() {
        let pd = make_box_mesh();
        let img = poly_data_to_image_data(&pd, [5, 5, 5]);
        let arr = img.point_data().get_array("Distance").unwrap();
        // Corner voxel 0,0,0 should be positive (outside with padding)
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] > 0.0,
            "corner distance should be positive (outside), got {}",
            buf[0]
        );
    }
}
