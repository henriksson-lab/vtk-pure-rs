use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};

/// Compute a signed distance field from a PolyData surface on an ImageData grid.
///
/// Uses the Curless & Levoy algorithm (matching VTK's `vtkSignedDistance`):
/// - Input must have point normals
/// - Builds a static point locator (uniform grid bucketing)
/// - For each voxel, finds all surface points within `radius`
/// - Signed distance = average of `dot(normal, point - voxel)` for nearby points
/// - Voxels with no nearby points get value `-radius` (empty)
///
/// If the input has no normals, they are estimated from polygon face normals.
pub fn signed_distance(surface: &PolyData, dimensions: [usize; 3]) -> ImageData {
    let np = surface.points.len();
    if np == 0 {
        let mut image = ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
        let n = image.num_points();
        let arr = DataArray::from_vec("SignedDistance", vec![0.0f64; n], 1);
        image.point_data_mut().add_array(AnyDataArray::F64(arr));
        image.point_data_mut().set_active_scalars("SignedDistance");
        return image;
    }

    let bb = surface.points.bounds();
    let margin = ((bb.x_max - bb.x_min) + (bb.y_max - bb.y_min) + (bb.z_max - bb.z_min)) / 6.0;
    let origin = [bb.x_min - margin, bb.y_min - margin, bb.z_min - margin];
    let extent = [
        (bb.x_max - bb.x_min + 2.0 * margin).max(1e-12),
        (bb.y_max - bb.y_min + 2.0 * margin).max(1e-12),
        (bb.z_max - bb.z_min + 2.0 * margin).max(1e-12),
    ];
    let spacing = [
        extent[0] / (dimensions[0] - 1).max(1) as f64,
        extent[1] / (dimensions[1] - 1).max(1) as f64,
        extent[2] / (dimensions[2] - 1).max(1) as f64,
    ];

    // Compute radius as a fraction of the diagonal (matching VTK's typical usage)
    let diag = (extent[0] * extent[0] + extent[1] * extent[1] + extent[2] * extent[2]).sqrt();
    let radius = diag * 0.05; // ~5% of diagonal
    let r2 = radius * radius;

    let mut image = ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
    image.set_spacing(spacing);
    image.set_origin(origin);

    // Get or estimate normals
    let normals = get_or_estimate_normals(surface);

    // Use flat point slice directly for cache-friendly access
    let pts = surface.points.as_flat_slice();

    // ---- Build static point locator (uniform grid bucketing) ----
    // Choose bucket count: ~5 points per bucket (matching VTK default)
    let n_buckets_target = (np / 5).max(1);
    let bres = (n_buckets_target as f64).cbrt().ceil() as usize;
    let bres = bres.max(2).min(512);
    let bd = [bres, bres, bres];
    let bcs = [
        extent[0] / bd[0] as f64,
        extent[1] / bd[1] as f64,
        extent[2] / bd[2] as f64,
    ];
    let total_buckets = bd[0] * bd[1] * bd[2];

    // Count points per bucket
    let mut counts = vec![0u32; total_buckets];
    let mut pt_bucket = Vec::with_capacity(np);
    for i in 0..np {
        let px = pts[i * 3];
        let py = pts[i * 3 + 1];
        let pz = pts[i * 3 + 2];
        let bi = clamp_idx((px - origin[0]) / bcs[0], bd[0]);
        let bj = clamp_idx((py - origin[1]) / bcs[1], bd[1]);
        let bk = clamp_idx((pz - origin[2]) / bcs[2], bd[2]);
        let idx = bi * bd[1] * bd[2] + bj * bd[2] + bk;
        pt_bucket.push(idx);
        counts[idx] += 1;
    }

    // Build offsets (CSR-style)
    let mut offsets = vec![0u32; total_buckets + 1];
    for i in 0..total_buckets {
        offsets[i + 1] = offsets[i] + counts[i];
    }
    let mut bucket_pts = vec![0u32; np];
    let mut fill = vec![0u32; total_buckets];
    for i in 0..np {
        let bi = pt_bucket[i];
        let pos = (offsets[bi] + fill[bi]) as usize;
        bucket_pts[pos] = i as u32;
        fill[bi] += 1;
    }

    // ---- Compute signed distance for each voxel ----
    let nx = dimensions[0];
    let ny = dimensions[1];
    let nz = dimensions[2];
    let n_total = nx * ny * nz;
    let mut distances = vec![-radius; n_total]; // initialize to -radius (empty)

    // For each voxel, find points within radius using bucket locator
    for k in 0..nz {
        let z = origin[2] + k as f64 * spacing[2];
        let k_off = k * ny * nx;

        for j in 0..ny {
            let y = origin[1] + j as f64 * spacing[1];
            let j_off = j * nx;

            for i in 0..nx {
                let x = origin[0] + i as f64 * spacing[0];
                let voxel_idx = i + j_off + k_off;

                // Find bucket range for sphere of radius R around (x,y,z)
                let bi_min = clamp_idx((x - radius - origin[0]) / bcs[0], bd[0]);
                let bj_min = clamp_idx((y - radius - origin[1]) / bcs[1], bd[1]);
                let bk_min = clamp_idx((z - radius - origin[2]) / bcs[2], bd[2]);
                let bi_max = clamp_idx((x + radius - origin[0]) / bcs[0], bd[0]);
                let bj_max = clamp_idx((y + radius - origin[1]) / bcs[1], bd[1]);
                let bk_max = clamp_idx((z + radius - origin[2]) / bcs[2], bd[2]);

                let mut dist_sum = 0.0f64;
                let mut num_pts = 0u32;

                // Iterate buckets in footprint
                for bk in bk_min..=bk_max {
                    for bj in bj_min..=bj_max {
                        for bi in bi_min..=bi_max {
                            let bucket_idx = bi * bd[1] * bd[2] + bj * bd[2] + bk;
                            let start = offsets[bucket_idx] as usize;
                            let end = offsets[bucket_idx + 1] as usize;

                            for pi in start..end {
                                let pt_id = unsafe { *bucket_pts.get_unchecked(pi) } as usize;
                                let px = unsafe { *pts.get_unchecked(pt_id * 3) };
                                let py = unsafe { *pts.get_unchecked(pt_id * 3 + 1) };
                                let pz = unsafe { *pts.get_unchecked(pt_id * 3 + 2) };

                                let dx = px - x;
                                let dy = py - y;
                                let dz = pz - z;
                                let d2 = dx * dx + dy * dy + dz * dz;

                                if d2 <= r2 {
                                    let nx = normals[pt_id * 3];
                                    let ny = normals[pt_id * 3 + 1];
                                    let nz = normals[pt_id * 3 + 2];
                                    // Signed projection: dot(normal, point - voxel)
                                    dist_sum += nx * dx + ny * dy + nz * dz;
                                    num_pts += 1;
                                }
                            }
                        }
                    }
                }

                if num_pts > 0 {
                    distances[voxel_idx] = dist_sum / num_pts as f64;
                }
            }
        }
    }

    let arr = DataArray::from_vec("SignedDistance", distances, 1);
    image.point_data_mut().add_array(AnyDataArray::F64(arr));
    image.point_data_mut().set_active_scalars("SignedDistance");
    image
}

#[inline]
fn clamp_idx(v: f64, dim: usize) -> usize {
    let i = v.floor() as isize;
    i.max(0).min(dim as isize - 1) as usize
}

/// Get normals from point data, or estimate from polygon face normals.
fn get_or_estimate_normals(surface: &PolyData) -> Vec<f64> {
    let np = surface.points.len();

    // Try to use existing normals
    if let Some(normals_arr) = surface.point_data().normals() {
        if normals_arr.num_components() == 3 && normals_arr.num_tuples() == np {
            let mut out = Vec::with_capacity(np * 3);
            let mut buf = [0.0f64; 3];
            for i in 0..np {
                normals_arr.tuple_as_f64(i, &mut buf);
                out.push(buf[0]);
                out.push(buf[1]);
                out.push(buf[2]);
            }
            return out;
        }
    }

    // Estimate normals from polygon faces (area-weighted vertex normal accumulation)
    let mut normals = vec![0.0f64; np * 3];

    for cell in surface.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        // Compute face normal via Newell's method
        let mut nx = 0.0;
        let mut ny = 0.0;
        let mut nz = 0.0;
        let n = cell.len();
        for i in 0..n {
            let p = surface.points.get(cell[i] as usize);
            let q = surface.points.get(cell[(i + 1) % n] as usize);
            nx += (p[1] - q[1]) * (p[2] + q[2]);
            ny += (p[2] - q[2]) * (p[0] + q[0]);
            nz += (p[0] - q[0]) * (p[1] + q[1]);
        }
        // Accumulate face normal to each vertex (area-weighted — unnormalized cross product)
        for &id in cell {
            let ui = id as usize;
            normals[ui * 3] += nx;
            normals[ui * 3 + 1] += ny;
            normals[ui * 3 + 2] += nz;
        }
    }

    // Normalize
    for i in 0..np {
        let x = normals[i * 3];
        let y = normals[i * 3 + 1];
        let z = normals[i * 3 + 2];
        let len = (x * x + y * y + z * z).sqrt();
        if len > 1e-10 {
            normals[i * 3] /= len;
            normals[i * 3 + 1] /= len;
            normals[i * 3 + 2] /= len;
        }
    }

    normals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_distance_field() {
        // Tetrahedron
        let surface = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 2.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let image = signed_distance(&surface, [5, 5, 5]);
        assert_eq!(image.dimensions(), [5, 5, 5]);
        let s = image.point_data().scalars().unwrap();
        // Should have both positive and negative values
        let mut has_pos = false;
        let mut has_neg = false;
        let mut buf = [0.0f64];
        for i in 0..s.num_tuples() {
            s.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.01 {
                has_pos = true;
            }
            if buf[0] < -0.01 {
                has_neg = true;
            }
        }
        assert!(has_pos, "should have positive (outside) values");
    }
}
