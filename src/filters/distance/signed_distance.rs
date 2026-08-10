use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};

/// VTK's default radius of influence of each point.
///
/// `vtkSignedDistance::vtkSignedDistance()` sets `this->Radius = 0.1`
/// (VTK/Filters/Points/vtkSignedDistance.cxx:153). Note this is an *absolute*
/// distance in world units, not a fraction of the model size, so callers
/// working at a different scale must pass their own value.
pub const DEFAULT_RADIUS: f64 = 0.1;

/// Compute a signed distance field from an oriented point cloud / surface on an
/// ImageData grid.
///
/// Port of VTK's `vtkSignedDistance` (VTK/Filters/Points/vtkSignedDistance.cxx),
/// which loosely follows Curless & Levoy, "A Volumetric Method for Building
/// Complex Models from Range Images".
///
/// The sampling volume is exactly the bounding box of the input points, with no
/// padding: the output origin is the bounds minimum and the spacing is
/// `(max - min) / (dim - 1)` per axis
/// (`vtkSignedDistance::StartAppend()`, cxx:203-209).
///
/// Every voxel is initialised to `-radius`, the "empty" value
/// (`StartAppend()`, cxx:181; see also the class docs in vtkSignedDistance.h:39-51).
/// For each voxel `x` the filter then finds all input points within `radius`
/// (a point locator, here a uniform-grid bucketing locator standing in for
/// `vtkStaticPointLocator`) and, if there is at least one, overwrites the voxel
/// with the *average* of `n_p · (p - x)` over those points
/// (`SignedDistanceFunctor::operator()`, cxx:100-113).
///
/// Because the projection is onto `p - x` (pointing from the voxel towards the
/// sample) rather than `x - p`, a voxel *inside* an outward-oriented surface
/// comes out **positive** and a voxel outside comes out **negative**
/// (cxx:110). Far-away voxels keep the empty value `-radius`.
///
/// `radius` corresponds to VTK's `SetRadius` (vtkSignedDistance.h:105); it is
/// clamped to be non-negative exactly like the `vtkSetClampMacro` there. Use
/// [`DEFAULT_RADIUS`] for VTK's default. Larger radii cost markedly more time.
///
/// Deviations from the C++, all deliberate:
/// - VTK requires float point normals on the input and errors out without them
///   (cxx:243-249); here missing normals are estimated from the polygon faces so
///   that plain triangle meshes can be used directly.
/// - VTK emits `float` scalars; this emits `f64` named `"SignedDistance"`.
pub fn signed_distance(surface: &PolyData, dimensions: [usize; 3], radius: f64) -> ImageData {
    signed_distance_with_bounds(surface, dimensions, radius, None)
}

/// As [`signed_distance`], but with an explicit sampling volume.
///
/// `bounds` is `[xmin, xmax, ymin, ymax, zmin, zmax]`, matching VTK's
/// `SetBounds` (vtkSignedDistance.h:94). A `None` — or a degenerate/inverted
/// box — falls back to the bounds of the input points, which is the same
/// validity test VTK applies in `StartAppend()` (cxx:190-200).
pub fn signed_distance_with_bounds(
    surface: &PolyData,
    dimensions: [usize; 3],
    radius: f64,
    bounds: Option<[f64; 6]>,
) -> ImageData {
    // vtkSetClampMacro(Radius, double, 0.0, VTK_FLOAT_MAX) — vtkSignedDistance.h:105
    let radius = if radius.is_finite() { radius.max(0.0) } else { 0.0 };
    let np = surface.points.len();

    // Model bounds: user supplied if valid, otherwise computed from the input
    // points (StartAppend, cxx:190-200).
    let valid = |b: &[f64; 6]| b[0] < b[1] && b[2] < b[3] && b[4] < b[5];
    let model_bounds = match bounds {
        Some(b) if valid(&b) => b,
        _ if np > 0 => {
            let bb = surface.points.bounds();
            [bb.x_min, bb.x_max, bb.y_min, bb.y_max, bb.z_min, bb.z_max]
        }
        // No points and no usable bounds: leave the volume at the origin with
        // unit spacing rather than propagating the empty bounding box.
        _ => [0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
    };

    // output->SetOrigin(Bounds[0], Bounds[2], Bounds[4]) — cxx:203
    let origin = [model_bounds[0], model_bounds[2], model_bounds[4]];
    // spacing[i] = (Bounds[2i+1] - Bounds[2i]) / (Dimensions[i] - 1) — cxx:205-209
    let spacing = [
        (model_bounds[1] - model_bounds[0]) / (dimensions[0].max(2) - 1) as f64,
        (model_bounds[3] - model_bounds[2]) / (dimensions[1].max(2) - 1) as f64,
        (model_bounds[5] - model_bounds[4]) / (dimensions[2].max(2) - 1) as f64,
    ];

    let nx = dimensions[0];
    let ny = dimensions[1];
    let nz = dimensions[2];

    let mut image = ImageData::with_dimensions(nx, ny, nz);
    image.set_spacing(spacing);
    image.set_origin(origin);

    // GetScalars()->Fill(-this->Radius) — cxx:181
    let n_total = image.num_points();
    let mut distances = vec![-radius; n_total];

    // Append() returns immediately for an empty input (cxx:228-231), leaving the
    // volume at the empty value.
    if np == 0 || n_total == 0 {
        return finish(image, distances);
    }

    // Get or estimate normals
    let normals = get_or_estimate_normals(surface);

    // Use flat point slice directly for cache-friendly access
    let pts = surface.points.as_flat_slice();

    // ---- Build the point locator (uniform grid bucketing) ----
    // Stands in for vtkStaticPointLocator, which bins the *input dataset*'s own
    // bounds independently of the sampling volume.
    let pb = surface.points.bounds();
    let loc_origin = [pb.x_min, pb.y_min, pb.z_min];
    let loc_extent = [
        (pb.x_max - pb.x_min).max(1e-12),
        (pb.y_max - pb.y_min).max(1e-12),
        (pb.z_max - pb.z_min).max(1e-12),
    ];
    // Choose bucket count: ~5 points per bucket.
    let n_buckets_target = (np / 5).max(1);
    let bres = (n_buckets_target as f64).cbrt().ceil() as usize;
    let bres = bres.clamp(2, 512);
    let bd = [bres, bres, bres];
    let bcs = [
        loc_extent[0] / bd[0] as f64,
        loc_extent[1] / bd[1] as f64,
        loc_extent[2] / bd[2] as f64,
    ];
    let total_buckets = bd[0] * bd[1] * bd[2];

    // Count points per bucket
    let mut counts = vec![0u32; total_buckets];
    let mut pt_bucket = Vec::with_capacity(np);
    for i in 0..np {
        let px = pts[i * 3];
        let py = pts[i * 3 + 1];
        let pz = pts[i * 3 + 2];
        let bi = clamp_idx((px - loc_origin[0]) / bcs[0], bd[0]);
        let bj = clamp_idx((py - loc_origin[1]) / bcs[1], bd[1]);
        let bk = clamp_idx((pz - loc_origin[2]) / bcs[2], bd[2]);
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

    // ---- Compute signed distance for each voxel (SignedDistanceFunctor, cxx:85-116) ----
    let r2 = radius * radius;

    for k in 0..nz {
        let z = origin[2] + k as f64 * spacing[2];
        let k_off = k * ny * nx;

        for j in 0..ny {
            let y = origin[1] + j as f64 * spacing[1];
            let j_off = j * nx;

            for i in 0..nx {
                let x = origin[0] + i as f64 * spacing[0];
                let voxel_idx = i + j_off + k_off;

                // Bucket footprint of the sphere of influence around (x,y,z).
                let bi_min = clamp_idx((x - radius - loc_origin[0]) / bcs[0], bd[0]);
                let bj_min = clamp_idx((y - radius - loc_origin[1]) / bcs[1], bd[1]);
                let bk_min = clamp_idx((z - radius - loc_origin[2]) / bcs[2], bd[2]);
                let bi_max = clamp_idx((x + radius - loc_origin[0]) / bcs[0], bd[0]);
                let bj_max = clamp_idx((y + radius - loc_origin[1]) / bcs[1], bd[1]);
                let bk_max = clamp_idx((z + radius - loc_origin[2]) / bcs[2], bd[2]);

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
                                    // dist += n[0]*(p[0]-x[0]) + ... — cxx:110
                                    dist_sum += nx * dx + ny * dy + nz * dz;
                                    num_pts += 1;
                                }
                            }
                        }
                    }
                }

                // Scalars[ptId] = dist / numPts, only when points were found — cxx:104-113
                if num_pts > 0 {
                    distances[voxel_idx] = dist_sum / num_pts as f64;
                }
            }
        }
    }

    finish(image, distances)
}

fn finish(mut image: ImageData, distances: Vec<f64>) -> ImageData {
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

    /// Outward-oriented UV sphere of unit radius centred at the origin.
    ///
    /// `vtkSignedDistance` is a *point cloud* filter: it averages
    /// `dot(normal, point - voxel)` over the surface samples inside `radius`, so
    /// the surface has to be sampled densely enough for voxels near it to see
    /// several samples. A bare tetrahedron (4 points) is not — every voxel either
    /// sees nothing or lands exactly on a vertex, where the projection is 0.
    fn uv_sphere(n_phi: usize, n_theta: usize) -> PolyData {
        use std::f64::consts::PI;
        let mut pts = Vec::new();
        for i in 0..=n_phi {
            let phi = PI * i as f64 / n_phi as f64;
            for j in 0..n_theta {
                let theta = 2.0 * PI * j as f64 / n_theta as f64;
                pts.push([
                    phi.sin() * theta.cos(),
                    phi.sin() * theta.sin(),
                    phi.cos(),
                ]);
            }
        }
        let idx = |i: usize, j: usize| (i * n_theta + j % n_theta) as i64;
        let mut tris = Vec::new();
        for i in 0..n_phi {
            for j in 0..n_theta {
                tris.push([idx(i, j), idx(i + 1, j), idx(i + 1, j + 1)]);
                tris.push([idx(i, j), idx(i + 1, j + 1), idx(i, j + 1)]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn signed_distance_field() {
        let surface = uv_sphere(24, 32);
        let image = signed_distance(&surface, [20, 20, 20], 0.25);
        assert_eq!(image.dimensions(), [20, 20, 20]);
        let s = image.point_data().scalars().unwrap();
        // The field must straddle zero: voxels on the two sides of the surface get
        // opposite-signed projections, and voxels too far away stay at the "empty"
        // value -radius.
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
        assert!(has_pos, "SDF should have positive values");
        assert!(has_neg, "SDF should have negative values");
    }

    /// The sampling volume is exactly the input bounding box: origin = bounds
    /// min and spacing = extent / (dim - 1), with no padding
    /// (vtkSignedDistance.cxx:203-209).
    #[test]
    fn signed_distance_volume_is_input_bounding_box() {
        let surface = uv_sphere(12, 16);
        let image = signed_distance(&surface, [11, 11, 11], DEFAULT_RADIUS);
        let origin = image.origin();
        let spacing = image.spacing();
        let bb = surface.points.bounds();
        for (o, b) in origin.iter().zip([bb.x_min, bb.y_min, bb.z_min]) {
            assert!((o - b).abs() < 1e-12, "origin must be the bounds minimum");
        }
        for (s, e) in spacing.iter().zip([
            bb.x_max - bb.x_min,
            bb.y_max - bb.y_min,
            bb.z_max - bb.z_min,
        ]) {
            assert!((s - e / 10.0).abs() < 1e-12, "spacing = extent / (dim - 1)");
        }
        // The far corner of the volume lands exactly on the bounds maximum.
        let far = origin[0] + 10.0 * spacing[0];
        assert!((far - bb.x_max).abs() < 1e-12);
    }

    /// Voxels that see no sample within `radius` keep the "empty" value -Radius
    /// (vtkSignedDistance.cxx:181).
    #[test]
    fn signed_distance_empty_voxels_are_negative_radius() {
        let surface = uv_sphere(24, 32);
        let radius = 0.2;
        let image = signed_distance(&surface, [21, 21, 21], radius);
        let s = image.point_data().scalars().unwrap();
        let dims = image.dimensions();
        let mut buf = [0.0f64];
        // The centre voxel of a unit sphere is 1.0 away from every sample, far
        // outside a radius of 0.2, so it must still hold -radius.
        let c = dims[0] / 2 + (dims[1] / 2) * dims[0] + (dims[2] / 2) * dims[0] * dims[1];
        s.tuple_as_f64(c, &mut buf);
        assert!(
            (buf[0] + radius).abs() < 1e-12,
            "unseen voxel should hold -radius, got {}",
            buf[0]
        );
        // And no voxel may ever go below -radius: values are either the empty
        // fill or an average of projections whose magnitude is <= radius.
        for i in 0..s.num_tuples() {
            s.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= -radius - 1e-12, "value {} below -radius", buf[0]);
        }
    }

    #[test]
    fn signed_distance_sign_convention_matches_vtk() {
        // vtkSignedDistance accumulates dot(normal, point - voxel), so a voxel on
        // the inside of an outward-oriented surface sees the samples "ahead" of it
        // along their own normals and comes out positive; an outside voxel is
        // negative. Probe a pair of voxels straddling the sphere along +x.
        //
        // The sampling volume is the input bounding box, whose faces touch the
        // sphere, so an explicit larger box is needed for exterior voxels to
        // exist at all along the probe row.
        let surface = uv_sphere(24, 32);
        let radius = 0.25;
        let image = signed_distance_with_bounds(
            &surface,
            [31, 31, 31],
            radius,
            Some([-1.5, 1.5, -1.5, 1.5, -1.5, 1.5]),
        );
        let s = image.point_data().scalars().unwrap();
        let dims = image.dimensions();
        let origin = image.origin();
        let spacing = image.spacing();
        let mut buf = [0.0f64];

        let mut inner = None;
        let mut outer = None;
        // Walk the grid row through the sphere centre along x.
        let j = ((0.0 - origin[1]) / spacing[1]).round() as usize;
        let k = ((0.0 - origin[2]) / spacing[2]).round() as usize;
        for i in 0..dims[0] {
            let x = origin[0] + i as f64 * spacing[0];
            let y = origin[1] + j as f64 * spacing[1];
            let z = origin[2] + k as f64 * spacing[2];
            let r = (x * x + y * y + z * z).sqrt();
            s.tuple_as_f64(i + j * dims[0] + k * dims[0] * dims[1], &mut buf);
            // Only look at voxels that actually saw surface samples, i.e. that
            // are not parked on the "empty" fill value of exactly -radius.
            if (buf[0] + radius).abs() < 1e-12 {
                continue;
            }
            if r < 1.0 && buf[0] > 0.01 {
                inner = Some(buf[0]);
            }
            if r > 1.0 && buf[0] < -0.01 {
                outer = Some(buf[0]);
            }
        }
        assert!(inner.is_some(), "interior voxels should be positive");
        assert!(outer.is_some(), "exterior voxels should be negative");
    }

    /// Radius is a user parameter, not derived from the model size
    /// (vtkSignedDistance.h:105, cxx:153).
    #[test]
    fn signed_distance_radius_is_absolute() {
        let surface = uv_sphere(24, 32);
        for radius in [0.1, 0.4] {
            let image = signed_distance(&surface, [15, 15, 15], radius);
            let s = image.point_data().scalars().unwrap();
            let mut min = f64::INFINITY;
            let mut buf = [0.0f64];
            for i in 0..s.num_tuples() {
                s.tuple_as_f64(i, &mut buf);
                min = min.min(buf[0]);
            }
            assert!(
                (min + radius).abs() < 1e-9,
                "empty fill must track the supplied radius {radius}, got min {min}"
            );
        }
    }

    /// A user-supplied box overrides the computed one; a degenerate one does not
    /// (vtkSignedDistance.cxx:190-200).
    #[test]
    fn signed_distance_honors_explicit_bounds() {
        let surface = uv_sphere(12, 16);
        let b = [-2.0, 2.0, -2.0, 2.0, -2.0, 2.0];
        let image = signed_distance_with_bounds(&surface, [5, 5, 5], 0.2, Some(b));
        assert_eq!(image.origin(), [-2.0, -2.0, -2.0]);
        assert_eq!(image.spacing(), [1.0, 1.0, 1.0]);

        // Inverted bounds are ignored in favour of the input bounds.
        let bad = [1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        let image = signed_distance_with_bounds(&surface, [5, 5, 5], 0.2, Some(bad));
        let bb = surface.points.bounds();
        assert!((image.origin()[0] - bb.x_min).abs() < 1e-12);
    }

    #[test]
    fn signed_distance_empty_input() {
        let image = signed_distance(&PolyData::new(), [4, 4, 4], 0.3);
        let s = image.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 64);
        let mut buf = [0.0f64];
        for i in 0..s.num_tuples() {
            s.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], -0.3);
        }
    }
}
