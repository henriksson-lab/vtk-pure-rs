//! Vector field topology analysis.
//!
//! Detects critical points (sources, sinks, saddles, centers) in 2D/3D vector
//! fields defined on ImageData. Analogous to VTK's vtkVectorFieldTopology.

use crate::data::{AnyDataArray, DataArray, ImageData, Points, PolyData};

/// Type of critical point in a vector field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CriticalPointType {
    /// Repelling node (all eigenvalues have positive real part)
    Source,
    /// Attracting node (all eigenvalues have negative real part)
    Sink,
    /// Saddle point (eigenvalues have mixed signs)
    Saddle,
    /// Center (purely imaginary eigenvalues — vortex core)
    Center,
    /// Degenerate or unclassified
    Degenerate,
}

impl std::fmt::Display for CriticalPointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source => write!(f, "Source"),
            Self::Sink => write!(f, "Sink"),
            Self::Saddle => write!(f, "Saddle"),
            Self::Center => write!(f, "Center"),
            Self::Degenerate => write!(f, "Degenerate"),
        }
    }
}

/// A detected critical point.
#[derive(Debug, Clone)]
pub struct CriticalPoint {
    pub position: [f64; 3],
    pub point_type: CriticalPointType,
    /// Jacobian eigenvalues (real parts only for 2D: [λ1_re, λ2_re])
    pub eigenvalues: Vec<f64>,
}

/// Find critical points in a 2D vector field on ImageData.
///
/// Searches for cells where the vector field changes sign (zero-crossing)
/// and classifies by the Jacobian at each location.
pub fn find_critical_points_2d(field: &ImageData) -> Vec<CriticalPoint> {
    let vectors = match field.point_data().vectors() {
        Some(v) if v.num_components() >= 2 => v,
        _ => return Vec::new(),
    };

    let dims = field.dimensions();
    let extent = field.extent();
    let spacing = field.spacing();
    let origin = field.origin();

    if dims[0] < 2 || dims[1] < 2 {
        return Vec::new();
    }

    let mut critical_points = Vec::new();

    // For each cell, check if vector field has a zero crossing
    for iy in 0..dims[1] - 1 {
        for ix in 0..dims[0] - 1 {
            let i00 = ix + iy * dims[0];
            let i10 = (ix + 1) + iy * dims[0];
            let i01 = ix + (iy + 1) * dims[0];
            let i11 = (ix + 1) + (iy + 1) * dims[0];

            let v00 = get_vec2(vectors, i00);
            let v10 = get_vec2(vectors, i10);
            let v01 = get_vec2(vectors, i01);
            let v11 = get_vec2(vectors, i11);

            // Check sign changes in both components
            let signs_x = [
                v00[0].signum(),
                v10[0].signum(),
                v01[0].signum(),
                v11[0].signum(),
            ];
            let signs_y = [
                v00[1].signum(),
                v10[1].signum(),
                v01[1].signum(),
                v11[1].signum(),
            ];

            let x_changes = signs_x.iter().any(|&s| s != signs_x[0]);
            let y_changes = signs_y.iter().any(|&s| s != signs_y[0]);

            if !x_changes || !y_changes {
                continue;
            }

            // Bilinear interpolation to find zero: solve v(x,y) ≈ 0
            // Use simple averaging as approximation
            let cx = origin[0] + (extent[0] as f64 + ix as f64 + 0.5) * spacing[0];
            let cy = origin[1] + (extent[2] as f64 + iy as f64 + 0.5) * spacing[1];

            // Compute Jacobian via finite differences at cell center
            let dvx_dx = ((v10[0] + v11[0]) - (v00[0] + v01[0])) / (2.0 * spacing[0]);
            let dvx_dy = ((v01[0] + v11[0]) - (v00[0] + v10[0])) / (2.0 * spacing[1]);
            let dvy_dx = ((v10[1] + v11[1]) - (v00[1] + v01[1])) / (2.0 * spacing[0]);
            let dvy_dy = ((v01[1] + v11[1]) - (v00[1] + v10[1])) / (2.0 * spacing[1]);

            // Classify by eigenvalues of 2x2 Jacobian
            let trace = dvx_dx + dvy_dy;
            let det = dvx_dx * dvy_dy - dvx_dy * dvy_dx;
            let discriminant = trace * trace - 4.0 * det;

            let point_type = if discriminant >= -1e-10 && discriminant <= 1e-10 {
                // Repeated real eigenvalues: both equal trace/2
                let lambda = trace / 2.0;
                if lambda > 1e-10 {
                    CriticalPointType::Source
                } else if lambda < -1e-10 {
                    CriticalPointType::Sink
                } else {
                    CriticalPointType::Center
                }
            } else if discriminant > 0.0 {
                let sqrt_d = discriminant.sqrt();
                let l1 = (trace + sqrt_d) / 2.0;
                let l2 = (trace - sqrt_d) / 2.0;
                if l1 > 0.0 && l2 > 0.0 {
                    CriticalPointType::Source
                } else if l1 < 0.0 && l2 < 0.0 {
                    CriticalPointType::Sink
                } else {
                    CriticalPointType::Saddle
                }
            } else {
                // Complex eigenvalues
                if trace.abs() < 1e-10 {
                    CriticalPointType::Center
                } else if trace > 0.0 {
                    CriticalPointType::Source // spiral source
                } else {
                    CriticalPointType::Sink // spiral sink
                }
            };

            let eigenvalues = if discriminant >= 0.0 {
                let sqrt_d = discriminant.sqrt();
                vec![(trace + sqrt_d) / 2.0, (trace - sqrt_d) / 2.0]
            } else {
                vec![trace / 2.0, trace / 2.0] // real part of complex pair
            };

            critical_points.push(CriticalPoint {
                position: [cx, cy, 0.0],
                point_type,
                eigenvalues,
            });
        }
    }

    critical_points
}

/// Convert critical points to a PolyData with vertex cells and type/eigenvalue data.
pub fn critical_points_to_poly_data(points: &[CriticalPoint]) -> PolyData {
    if points.is_empty() {
        return PolyData::new();
    }

    let mut pts = Points::<f64>::new();
    let mut types = Vec::with_capacity(points.len());
    let mut ev1 = Vec::with_capacity(points.len());
    let mut ev2 = Vec::with_capacity(points.len());

    for cp in points {
        pts.push(cp.position);
        types.push(match cp.point_type {
            CriticalPointType::Source => 0.0,
            CriticalPointType::Sink => 1.0,
            CriticalPointType::Saddle => 2.0,
            CriticalPointType::Center => 3.0,
            CriticalPointType::Degenerate => 4.0,
        });
        ev1.push(if !cp.eigenvalues.is_empty() {
            cp.eigenvalues[0]
        } else {
            0.0
        });
        ev2.push(if cp.eigenvalues.len() > 1 {
            cp.eigenvalues[1]
        } else {
            0.0
        });
    }

    let mut mesh = PolyData::new();
    mesh.points = pts;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CriticalPointType",
            types,
            1,
        )));
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Eigenvalue1",
            ev1,
            1,
        )));
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Eigenvalue2",
            ev2,
            1,
        )));
    mesh
}

/// Compute vorticity magnitude (curl magnitude) of a 3D vector field on ImageData.
///
/// Adds a "Vorticity" scalar array to point data.
pub fn compute_vorticity(field: &ImageData) -> ImageData {
    let vectors = match field.point_data().vectors() {
        Some(v) if v.num_components() == 3 => v,
        _ => return field.clone(),
    };

    let dims = field.dimensions();
    let spacing = field.spacing();
    let n = dims[0] * dims[1] * dims[2];

    let mut vorticity = vec![0.0f64; n];

    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let idx = ix + iy * dims[0] + iz * dims[0] * dims[1];

                // Central differences with clamped boundary
                let ixm = if ix > 0 { ix - 1 } else { ix };
                let ixp = if ix + 1 < dims[0] { ix + 1 } else { ix };
                let iym = if iy > 0 { iy - 1 } else { iy };
                let iyp = if iy + 1 < dims[1] { iy + 1 } else { iy };
                let izm = if iz > 0 { iz - 1 } else { iz };
                let izp = if iz + 1 < dims[2] { iz + 1 } else { iz };

                let dx = (ixp - ixm).max(1) as f64 * spacing[0];
                let dy = (iyp - iym).max(1) as f64 * spacing[1];
                let dz = (izp - izm).max(1) as f64 * spacing[2];

                let vxp = get_vec3(vectors, ixp + iy * dims[0] + iz * dims[0] * dims[1]);
                let vxm = get_vec3(vectors, ixm + iy * dims[0] + iz * dims[0] * dims[1]);
                let vyp = get_vec3(vectors, ix + iyp * dims[0] + iz * dims[0] * dims[1]);
                let vym = get_vec3(vectors, ix + iym * dims[0] + iz * dims[0] * dims[1]);
                let vzp = get_vec3(vectors, ix + iy * dims[0] + izp * dims[0] * dims[1]);
                let vzm = get_vec3(vectors, ix + iy * dims[0] + izm * dims[0] * dims[1]);

                // curl = (dw/dy - dv/dz, du/dz - dw/dx, dv/dx - du/dy)
                let curl_x = (vzp[1] - vzm[1]) / dz - (vyp[2] - vym[2]) / dy;
                let curl_y = (vxp[2] - vxm[2]) / dx - (vzp[0] - vzm[0]) / dz;
                let curl_z = (vyp[0] - vym[0]) / dy - (vxp[1] - vxm[1]) / dx;

                vorticity[idx] = (curl_x * curl_x + curl_y * curl_y + curl_z * curl_z).sqrt();
            }
        }
    }

    let mut result = field.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Vorticity",
            vorticity,
            1,
        )));
    result
}

fn get_vec2(arr: &AnyDataArray, idx: usize) -> [f64; 2] {
    let mut buf = [0.0f64; 3];
    arr.tuple_as_f64(idx, &mut buf);
    [buf[0], buf[1]]
}

fn get_vec3(arr: &AnyDataArray, idx: usize) -> [f64; 3] {
    let mut buf = [0.0f64; 3];
    arr.tuple_as_f64(idx, &mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source_field() -> ImageData {
        // Radial outward flow: v = (x - cx, y - cy, 0) — source at center
        let dims = [20, 20, 1];
        let spacing = [0.1, 0.1, 1.0];
        let origin = [0.0, 0.0, 0.0];
        let cx = 1.0;
        let cy = 1.0;

        let n = dims[0] * dims[1] * dims[2];
        let mut vdata = Vec::with_capacity(n * 3);
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let x = origin[0] + ix as f64 * spacing[0];
                let y = origin[1] + iy as f64 * spacing[1];
                vdata.push(x - cx);
                vdata.push(y - cy);
                vdata.push(0.0);
            }
        }

        let mut field = ImageData::new();
        field.set_extent([
            0,
            dims[0] as i64 - 1,
            0,
            dims[1] as i64 - 1,
            0,
            dims[2] as i64 - 1,
        ]);
        field.set_spacing(spacing);
        field.set_origin(origin);
        field
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("velocity", vdata, 3)));
        field.point_data_mut().set_active_vectors("velocity");
        field
    }

    #[test]
    fn find_source() {
        let field = make_source_field();
        let cps = find_critical_points_2d(&field);
        assert!(!cps.is_empty());
        // Should find a source near (1.0, 1.0)
        let source = cps
            .iter()
            .find(|cp| cp.point_type == CriticalPointType::Source);
        assert!(
            source.is_some(),
            "expected a source point, found: {:?}",
            cps.iter().map(|cp| &cp.point_type).collect::<Vec<_>>()
        );
    }

    #[test]
    fn critical_point_position_respects_extent() {
        let mut field = make_source_field();
        field.set_extent([10, 29, 20, 39, 0, 0]);
        field.set_origin([-1.0, -2.0, 0.0]);

        let cps = find_critical_points_2d(&field);
        let source = cps
            .iter()
            .find(|cp| cp.point_type == CriticalPointType::Source)
            .expect("expected source point");

        assert!(source.position[0] > 0.0);
        assert!(source.position[1] > 0.0);
    }

    #[test]
    fn sink_field() {
        // Inward flow: v = -(x - cx, y - cy, 0)
        let dims = [20, 20, 1];
        let spacing = [0.1, 0.1, 1.0];
        let origin = [0.0, 0.0, 0.0];

        let n = dims[0] * dims[1];
        let mut vdata = Vec::with_capacity(n * 3);
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                let x = origin[0] + ix as f64 * spacing[0];
                let y = origin[1] + iy as f64 * spacing[1];
                vdata.push(-(x - 1.0));
                vdata.push(-(y - 1.0));
                vdata.push(0.0);
            }
        }

        let mut field = ImageData::new();
        field.set_extent([
            0,
            dims[0] as i64 - 1,
            0,
            dims[1] as i64 - 1,
            0,
            dims[2] as i64 - 1,
        ]);
        field.set_spacing(spacing);
        field.set_origin(origin);
        field
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("velocity", vdata, 3)));
        field.point_data_mut().set_active_vectors("velocity");

        let cps = find_critical_points_2d(&field);
        let sink = cps
            .iter()
            .find(|cp| cp.point_type == CriticalPointType::Sink);
        assert!(sink.is_some());
    }

    #[test]
    fn critical_points_to_mesh() {
        let field = make_source_field();
        let cps = find_critical_points_2d(&field);
        let mesh = critical_points_to_poly_data(&cps);
        assert_eq!(mesh.points.len(), cps.len());
        assert!(mesh.point_data().get_array("CriticalPointType").is_some());
    }

    #[test]
    fn vorticity_computation() {
        // Solid body rotation: v = (-y, x, 0) — constant vorticity
        let dims = [10, 10, 10];
        let spacing = [0.1, 0.1, 0.1];
        let origin = [0.0, 0.0, 0.0];

        let n = dims[0] * dims[1] * dims[2];
        let mut vdata = Vec::with_capacity(n * 3);
        for _iz in 0..dims[2] {
            for iy in 0..dims[1] {
                for ix in 0..dims[0] {
                    let x = origin[0] + ix as f64 * spacing[0];
                    let y = origin[1] + iy as f64 * spacing[1];
                    vdata.push(-y);
                    vdata.push(x);
                    vdata.push(0.0);
                }
            }
        }

        let mut field = ImageData::new();
        field.set_extent([
            0,
            dims[0] as i64 - 1,
            0,
            dims[1] as i64 - 1,
            0,
            dims[2] as i64 - 1,
        ]);
        field.set_spacing(spacing);
        field.set_origin(origin);
        field
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("velocity", vdata, 3)));
        field.point_data_mut().set_active_vectors("velocity");

        let result = compute_vorticity(&field);
        assert!(result.point_data().get_array("Vorticity").is_some());

        // Interior points should have vorticity ≈ 2 (curl of (-y,x,0) = (0,0,2))
        let vort = result.point_data().get_array("Vorticity").unwrap();
        let mut buf = [0.0f64];
        // Check an interior point
        let idx = 5 + 5 * dims[0] + 5 * dims[0] * dims[1];
        vort.tuple_as_f64(idx, &mut buf);
        assert!(
            (buf[0] - 2.0).abs() < 0.5,
            "vorticity at interior: {}",
            buf[0]
        );
    }

    #[test]
    fn empty_field() {
        let field = ImageData::new();
        let cps = find_critical_points_2d(&field);
        assert!(cps.is_empty());
    }
}
