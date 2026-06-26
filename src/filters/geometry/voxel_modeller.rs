use crate::data::PolyData;
use crate::data::{AnyDataArray, DataArray, DataSet, ImageData};

/// Convert a PolyData surface to a binary voxel volume (ImageData).
///
/// Produces an ImageData where each voxel is 1.0 if any triangle intersects
/// the voxel cell around the sample point, and 0.0 otherwise.
///
/// `max_distance` matches VTK's `MaximumDistance`: a fraction of the largest
/// input extent used to pad the automatically computed model bounds.
pub fn voxel_modeller(input: &PolyData, dimensions: [usize; 3], max_distance: f64) -> ImageData {
    if input.points.is_empty() {
        let mut image = ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
        let scalars = vec![0.0f64; image.num_points()];
        let arr = DataArray::from_vec("voxels", scalars, 1);
        image.point_data_mut().add_array(AnyDataArray::F64(arr));
        image.point_data_mut().set_active_scalars("voxels");
        return image;
    }

    let bb = input.points.bounds();
    let max_extent = (bb.x_max - bb.x_min)
        .max(bb.y_max - bb.y_min)
        .max(bb.z_max - bb.z_min);
    let margin = max_distance.clamp(0.0, 1.0) * max_extent;
    let origin = [bb.x_min - margin, bb.y_min - margin, bb.z_min - margin];
    let spacing = [
        (bb.x_max - bb.x_min + 2.0 * margin) / (dimensions[0] - 1).max(1) as f64,
        (bb.y_max - bb.y_min + 2.0 * margin) / (dimensions[1] - 1).max(1) as f64,
        (bb.z_max - bb.z_min + 2.0 * margin) / (dimensions[2] - 1).max(1) as f64,
    ];

    let mut image = ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
    image.set_spacing(spacing);
    image.set_origin(origin);

    let n_points = image.num_points();
    let voxel_half_width = [spacing[0] / 2.0, spacing[1] / 2.0, spacing[2] / 2.0];

    // Collect triangle vertices for distance computation
    let mut tris: Vec<([f64; 3], [f64; 3], [f64; 3])> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() >= 3 {
            let p0 = input.points.get(cell[0] as usize);
            for i in 1..cell.len() - 1 {
                let p1 = input.points.get(cell[i] as usize);
                let p2 = input.points.get(cell[i + 1] as usize);
                tris.push((p0, p1, p2));
            }
        }
    }

    let mut scalars = vec![0.0f64; n_points];
    for (idx, scalar) in scalars.iter_mut().enumerate() {
        let p = image.point(idx);
        for &(v0, v1, v2) in &tris {
            let closest = closest_point_on_triangle(p, v0, v1, v2);
            if (closest[0] - p[0]).abs() <= voxel_half_width[0] + 1e-12
                && (closest[1] - p[1]).abs() <= voxel_half_width[1] + 1e-12
                && (closest[2] - p[2]).abs() <= voxel_half_width[2] + 1e-12
            {
                *scalar = 1.0;
                break;
            }
        }
    }

    let arr = DataArray::from_vec("voxels", scalars, 1);
    image.point_data_mut().add_array(AnyDataArray::F64(arr));
    image.point_data_mut().set_active_scalars("voxels");
    image
}

fn closest_point_on_triangle(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
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

    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voxelize_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let image = voxel_modeller(&pd, [10, 10, 10], 0.2);
        assert_eq!(image.dimensions(), [10, 10, 10]);
        let s = image.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 1000);
        // Some voxels should be 1.0
        let mut has_ones = false;
        let mut buf = [0.0f64];
        for i in 0..s.num_tuples() {
            s.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                has_ones = true;
                break;
            }
        }
        assert!(has_ones);
    }
}
