use crate::data::{AnyDataArray, DataArray, DataSet, ImageData, PolyData};

/// Compute a distance field from a PolyData surface on an ImageData grid.
///
/// Each voxel stores the minimum unsigned distance to any triangle in the
/// input surface. This is similar to `voxel_modeller` but produces a
/// continuous distance field instead of a binary volume.
pub fn implicit_modeller(input: &PolyData, dimensions: [usize; 3]) -> ImageData {
    let bb = input.points.bounds();
    let margin = ((bb.x_max - bb.x_min) + (bb.y_max - bb.y_min) + (bb.z_max - bb.z_min)) / 6.0;
    let origin = [bb.x_min - margin, bb.y_min - margin, bb.z_min - margin];
    let spacing = [
        (bb.x_max - bb.x_min + 2.0 * margin) / (dimensions[0] - 1).max(1) as f64,
        (bb.y_max - bb.y_min + 2.0 * margin) / (dimensions[1] - 1).max(1) as f64,
        (bb.z_max - bb.z_min + 2.0 * margin) / (dimensions[2] - 1).max(1) as f64,
    ];

    let mut image = ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
    image.set_spacing(spacing);
    image.set_origin(origin);

    // Collect triangles
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3])> = input
        .polys
        .iter()
        .flat_map(|cell| {
            let p0 = input.points.get(cell[0] as usize);
            (1..cell.len() - 1).map(move |i| {
                (
                    p0,
                    input.points.get(cell[i] as usize),
                    input.points.get(cell[i + 1] as usize),
                )
            })
        })
        .collect();

    let n = image.num_points();
    let mut distances = Vec::with_capacity(n);
    for i in 0..n {
        let p = image.point(i);
        let mut min_d2 = f64::MAX;
        for &(a, b, c) in &tris {
            let d2 = point_triangle_dist2(p, a, b, c);
            if d2 < min_d2 {
                min_d2 = d2;
            }
        }
        distances.push(min_d2.sqrt());
    }

    let arr = DataArray::from_vec("Distance", distances, 1);
    image.point_data_mut().add_array(AnyDataArray::F64(arr));
    image.point_data_mut().set_active_scalars("Distance");
    image
}

fn point_triangle_dist2(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return dist2(p, a);
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return dist2(p, b);
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return dist2(p, c);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return dist2(p, [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]]);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return dist2(p, [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]]);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return dist2(
            p,
            [
                b[0] + w * (c[0] - b[0]),
                b[1] + w * (c[1] - b[1]),
                b[2] + w * (c[2] - b[2]),
            ],
        );
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    dist2(
        p,
        [
            a[0] + ab[0] * v + ac[0] * w,
            a[1] + ab[1] * v + ac[1] * w,
            a[2] + ab[2] * v + ac[2] * w,
        ],
    )
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_field_from_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let image = implicit_modeller(&pd, [5, 5, 5]);
        assert_eq!(image.dimensions(), [5, 5, 5]);
        let s = image.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 125);
        // All distances should be non-negative
        let mut buf = [0.0f64];
        for i in 0..s.num_tuples() {
            s.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 0.0);
        }
    }
}
