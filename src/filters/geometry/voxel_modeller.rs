use crate::data::PolyData;
use crate::data::{AnyDataArray, DataArray, DataSet, ImageData};

/// Convert a PolyData surface to a binary voxel volume (ImageData).
///
/// Produces an ImageData where each voxel is 1.0 if its center is within
/// `max_distance` of any triangle in the input, and 0.0 otherwise.
/// This is a simple distance-based voxelization.
pub fn voxel_modeller(input: &PolyData, dimensions: [usize; 3], max_distance: f64) -> ImageData {
    let bb = input.points.bounds();
    let margin = max_distance * 2.0;
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
    let max_dist2 = max_distance * max_distance;

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
        let mut min_dist2 = f64::MAX;
        for &(v0, v1, v2) in &tris {
            let d2 = point_triangle_dist2(p, v0, v1, v2);
            if d2 < min_dist2 {
                min_dist2 = d2;
            }
            if min_dist2 <= max_dist2 {
                break;
            }
        }
        if min_dist2 <= max_dist2 {
            *scalar = 1.0;
        }
    }

    let arr = DataArray::from_vec("voxels", scalars, 1);
    image.point_data_mut().add_array(AnyDataArray::F64(arr));
    image.point_data_mut().set_active_scalars("voxels");
    image
}

fn point_triangle_dist2(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    // Project point onto triangle plane and find closest point
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
        let proj = [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
        return dist2(p, proj);
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let proj = [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
        return dist2(p, proj);
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let proj = [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
        return dist2(p, proj);
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let proj = [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ];
    dist2(p, proj)
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
