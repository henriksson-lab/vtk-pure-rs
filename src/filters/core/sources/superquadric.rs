use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

const MAX_SUPERQUADRIC_RESOLUTION: usize = 1024;
const MIN_SUPERQUADRIC_THICKNESS: f64 = 1e-4;
const MIN_SUPERQUADRIC_ROUNDNESS: f64 = 1e-24;
const SQ_SMALL_OFFSET: f64 = 0.01;

/// Parameters for generating a superquadric (superellipsoid).
pub struct SuperquadricParams {
    /// Whether to generate a toroidal superquadric.
    pub toroidal: bool,
    /// Axis of symmetry: x axis = 0, y axis = 1, z axis = 2. Default: 1.
    pub axis_of_symmetry: i32,
    /// Ring thickness for toroidal superquadrics. Default: 0.3333.
    pub thickness: f64,
    /// Isotropic size. Default: 0.5.
    pub size: f64,
    /// Phi roundness exponent. Default: 1.0 (sphere-like)
    pub phi_roundness: f64,
    /// Theta roundness exponent. Default: 1.0 (sphere-like)
    pub theta_roundness: f64,
    /// Scale factors [x, y, z]. Default: [1, 1, 1]
    pub scale: [f64; 3],
    /// Center. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Resolution in theta direction. Default: 16
    pub theta_resolution: usize,
    /// Resolution in phi direction. Default: 16
    pub phi_resolution: usize,
}

impl Default for SuperquadricParams {
    fn default() -> Self {
        Self {
            toroidal: false,
            axis_of_symmetry: 1,
            thickness: 0.3333,
            size: 0.5,
            phi_roundness: 1.0,
            theta_roundness: 1.0,
            scale: [1.0, 1.0, 1.0],
            center: [0.0, 0.0, 0.0],
            theta_resolution: 16,
            phi_resolution: 16,
        }
    }
}

/// Generate a superquadric surface following VTK's `vtkSuperquadricSource`.
///
/// The parametric equations use signed power functions to produce a variety
/// of shapes: spheres, cubes (rounded), cylinders, octrahedra, etc.
pub fn superquadric(params: &SuperquadricParams) -> PolyData {
    let theta_resolution = set_theta_resolution(params.theta_resolution);
    let phi_resolution = set_phi_resolution(params.phi_resolution);

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut tcoords = DataArray::<f64>::new("TextureCoords", 2);
    let mut polys = CellArray::new();

    let mut dims = [
        params.scale[0] * params.size,
        params.scale[1] * params.size,
        params.scale[2] * params.size,
    ];
    let (phi_lim, theta_lim, alpha) = if params.toroidal {
        let alpha = 1.0 / params.thickness.clamp(MIN_SUPERQUADRIC_THICKNESS, 1.0);
        let scale = alpha + 1.0;
        dims[0] /= scale;
        dims[1] /= scale;
        dims[2] /= scale;
        ([-PI, PI], [-PI, PI], alpha)
    } else {
        ([-PI / 2.0, PI / 2.0], [-PI, PI], 0.0)
    };

    let theta_roundness = params.theta_roundness.max(MIN_SUPERQUADRIC_ROUNDNESS);
    let phi_roundness = params.phi_roundness.max(MIN_SUPERQUADRIC_ROUNDNESS);
    let delta_phi = (phi_lim[1] - phi_lim[0]) / phi_resolution as f64;
    let delta_phi_tex = 1.0 / phi_resolution as f64;
    let delta_theta = (theta_lim[1] - theta_lim[0]) / theta_resolution as f64;
    let delta_theta_tex = 1.0 / theta_resolution as f64;

    let phi_segs = 4;
    let theta_segs = 8;
    let phi_subsegs = phi_resolution / phi_segs;
    let theta_subsegs = theta_resolution / theta_segs;

    for iq in 0..phi_segs {
        for i in 0..=phi_subsegs {
            let phi_index = i + iq * phi_subsegs;
            let phi = phi_lim[0] + delta_phi * phi_index as f64;
            let tcoord_y = delta_phi_tex * phi_index as f64;
            let phi_offset = if i == 0 {
                SQ_SMALL_OFFSET * delta_phi
            } else if i == phi_subsegs {
                -SQ_SMALL_OFFSET * delta_phi
            } else {
                0.0
            };

            for jq in 0..theta_segs {
                for j in 0..=theta_subsegs {
                    let theta_index = j + jq * theta_subsegs;
                    let theta = theta_lim[0] + delta_theta * theta_index as f64;
                    let tcoord_x = delta_theta_tex * theta_index as f64;
                    let theta_offset = if j == 0 {
                        SQ_SMALL_OFFSET * delta_theta
                    } else if j == theta_subsegs {
                        -SQ_SMALL_OFFSET * delta_theta
                    } else {
                        0.0
                    };

                    let (mut point, mut normal) = eval_superquadric(
                        theta,
                        phi,
                        theta_offset,
                        phi_offset,
                        theta_roundness,
                        phi_roundness,
                        dims,
                        alpha,
                    );
                    apply_axis_of_symmetry(params.axis_of_symmetry, &mut point, &mut normal);

                    let len =
                        (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                            .sqrt();
                    if len != 0.0 {
                        normal[0] /= len;
                        normal[1] /= len;
                        normal[2] /= len;
                    }

                    if !params.toroidal
                        && ((iq == 0 && i == 0) || (iq == phi_segs - 1 && i == phi_subsegs))
                    {
                        match params.axis_of_symmetry {
                            0 => {
                                point[1] = 0.0;
                                point[2] = 0.0;
                            }
                            1 => {
                                point[0] = 0.0;
                                point[2] = 0.0;
                            }
                            _ => {
                                point[0] = 0.0;
                                point[1] = 0.0;
                            }
                        }
                    }

                    point[0] += params.center[0];
                    point[1] += params.center[1];
                    point[2] += params.center[2];

                    points.push(point);
                    normals.push_tuple(&normal);
                    tcoords.push_tuple(&[tcoord_x, tcoord_y]);
                }
            }
        }
    }

    let row_offset = theta_resolution + theta_segs;
    for iq in 0..phi_segs {
        for i in 0..phi_subsegs {
            let pbase = row_offset * (i + iq * (phi_subsegs + 1));
            for jq in 0..theta_segs {
                let base = pbase + jq * (theta_subsegs + 1);
                let mut strip = Vec::with_capacity(theta_subsegs * 2 + 2);
                for j in 0..=theta_subsegs {
                    strip.push((base + row_offset + j) as i64);
                    strip.push((base + j) as i64);
                }
                polys.push_cell(&strip);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.strips = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut().add_array(tcoords.into());
    pd.point_data_mut().set_active_tcoords("TextureCoords");
    pd
}

fn set_phi_resolution(i: usize) -> usize {
    i.max(4)
        .div_ceil(4)
        .saturating_mul(4)
        .min(MAX_SUPERQUADRIC_RESOLUTION)
}

fn set_theta_resolution(i: usize) -> usize {
    i.max(8)
        .div_ceil(8)
        .saturating_mul(8)
        .min(MAX_SUPERQUADRIC_RESOLUTION)
}

fn cf(w: f64, m: f64, a: f64) -> f64 {
    let c = if w == PI || w == -PI { -1.0 } else { w.cos() };
    let sgn = if c < 0.0 { -1.0 } else { 1.0 };
    a + sgn * (sgn * c).powf(m)
}

fn sf(w: f64, m: f64) -> f64 {
    let s = if w == PI || w == -PI { 0.0 } else { w.sin() };
    let sgn = if s < 0.0 { -1.0 } else { 1.0 };
    sgn * (sgn * s).powf(m)
}

#[allow(clippy::too_many_arguments)]
fn eval_superquadric(
    theta: f64,
    phi: f64,
    dtheta: f64,
    dphi: f64,
    rtheta: f64,
    rphi: f64,
    dims: [f64; 3],
    alpha: f64,
) -> ([f64; 3], [f64; 3]) {
    let cf1 = cf(phi, rphi, alpha);
    let point = [
        -dims[0] * cf1 * sf(theta, rtheta),
        dims[1] * cf1 * cf(theta, rtheta, 0.0),
        dims[2] * sf(phi, rphi),
    ];

    let cf2 = cf(phi + dphi, 2.0 - rphi, 0.0);
    let normal = [
        -cf2 * sf(theta + dtheta, 2.0 - rtheta) / dims[0],
        cf2 * cf(theta + dtheta, 2.0 - rtheta, 0.0) / dims[1],
        sf(phi + dphi, 2.0 - rphi) / dims[2],
    ];

    (point, normal)
}

fn apply_axis_of_symmetry(axis_of_symmetry: i32, point: &mut [f64; 3], normal: &mut [f64; 3]) {
    match axis_of_symmetry {
        0 => {
            point.swap(0, 2);
            point[1] = -point[1];
            normal.swap(0, 2);
            normal[1] = -normal[1];
        }
        1 => {
            point.swap(1, 2);
            point[0] = -point[0];
            normal.swap(1, 2);
            normal[0] = -normal[0];
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_superquadric() {
        let pd = superquadric(&SuperquadricParams::default());
        assert_eq!(pd.points.len(), 480);
        assert_eq!(pd.strips.num_cells(), 128);
    }

    #[test]
    fn boxy_superquadric() {
        let pd = superquadric(&SuperquadricParams {
            phi_roundness: 0.1,
            theta_roundness: 0.1,
            ..Default::default()
        });
        assert!(pd.points.len() > 0);
        assert!(pd.strips.num_cells() > 0);
    }
}
