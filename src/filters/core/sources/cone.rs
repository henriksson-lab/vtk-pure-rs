use std::f64::consts::PI;

use crate::data::{CellArray, Points, PolyData};

/// Parameters for generating a cone.
pub struct ConeParams {
    pub center: [f64; 3],
    pub height: f64,
    pub radius: f64,
    pub direction: [f64; 3],
    pub resolution: usize,
    pub capping: bool,
}

impl Default for ConeParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            height: 1.0,
            radius: 0.5,
            direction: [1.0, 0.0, 0.0],
            resolution: 6,
            capping: true,
        }
    }
}

/// Generate a cone as PolyData.
///
/// The cone axis runs along `direction` with the apex at `center + direction * height/2`
/// and the base at `center - direction * height/2`.
pub fn cone(params: &ConeParams) -> PolyData {
    let mut points = Points::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let resolution = params.resolution;
    let angle = if resolution != 0 {
        2.0 * PI / resolution as f64
    } else {
        0.0
    };
    let xbot = -params.height / 2.0;

    points.push(transform_point(params, [params.height / 2.0, 0.0, 0.0]));

    match resolution {
        0 => {
            points.push(transform_point(params, [xbot, 0.0, 0.0]));
            lines.push_cell(&[0, 1]);
        }
        1 => {
            points.push(transform_point(params, [xbot, -params.radius, 0.0]));
            points.push(transform_point(params, [xbot, params.radius, 0.0]));
            polys.push_cell(&[0, 1, 2]);
        }
        2 => {
            points.push(transform_point(params, [xbot, 0.0, -params.radius]));
            points.push(transform_point(params, [xbot, 0.0, params.radius]));
            polys.push_cell(&[0, 1, 2]);
            points.push(transform_point(params, [xbot, -params.radius, 0.0]));
            points.push(transform_point(params, [xbot, params.radius, 0.0]));
            polys.push_cell(&[0, 3, 4]);
        }
        _ => {
            if params.capping {
                let mut cap = Vec::with_capacity(resolution);
                for i in 0..resolution {
                    let p = transform_point(
                        params,
                        [
                            xbot,
                            params.radius * (i as f64 * angle).cos(),
                            params.radius * (i as f64 * angle).sin(),
                        ],
                    );
                    points.push(p);
                    cap.push((resolution - i) as i64);
                }
                polys.push_cell(&cap);
                for i in 0..resolution {
                    let mut next = i + 2;
                    if next > resolution {
                        next = 1;
                    }
                    polys.push_cell(&[0, (i + 1) as i64, next as i64]);
                }
            } else {
                let first = transform_point(params, [xbot, params.radius, 0.0]);
                points.push(first);
                let mut previous = 1;
                for i in 0..resolution {
                    let p = transform_point(
                        params,
                        [
                            xbot,
                            params.radius * ((i + 1) as f64 * angle).cos(),
                            params.radius * ((i + 1) as f64 * angle).sin(),
                        ],
                    );
                    points.push(p);
                    let current = points.len() as i64 - 1;
                    polys.push_cell(&[0, previous, current]);
                    previous = current;
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd.polys = polys;
    pd
}

fn transform_point(params: &ConeParams, point: [f64; 3]) -> [f64; 3] {
    let mut x = point;

    if params.center != [0.0, 0.0, 0.0] || params.direction != [1.0, 0.0, 0.0] {
        let v_mag = (params.direction[0] * params.direction[0]
            + params.direction[1] * params.direction[1]
            + params.direction[2] * params.direction[2])
            .sqrt();

        if params.direction[0] < 0.0 {
            x = rotate_wxyz_180(x, [0.0, 1.0, 0.0]);
            x = rotate_wxyz_180(
                x,
                [
                    (params.direction[0] - v_mag) / 2.0,
                    params.direction[1] / 2.0,
                    params.direction[2] / 2.0,
                ],
            );
        } else {
            x = rotate_wxyz_180(
                x,
                [
                    (params.direction[0] + v_mag) / 2.0,
                    params.direction[1] / 2.0,
                    params.direction[2] / 2.0,
                ],
            );
        }
    }

    [
        x[0] + params.center[0],
        x[1] + params.center[1],
        x[2] + params.center[2],
    ]
}

fn rotate_wxyz_180(x: [f64; 3], axis: [f64; 3]) -> [f64; 3] {
    let norm = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if norm == 0.0 {
        return x;
    }

    let u = [axis[0] / norm, axis[1] / norm, axis[2] / norm];
    let dot = u[0] * x[0] + u[1] * x[1] + u[2] * x[2];

    [
        2.0 * u[0] * dot - x[0],
        2.0 * u[1] * dot - x[1],
        2.0 * u[2] * dot - x[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cone() {
        let pd = cone(&ConeParams::default());
        assert!(pd.points.len() > 0);
        assert!(pd.polys.num_cells() > 0);
    }

    #[test]
    fn cone_no_cap() {
        let pd = cone(&ConeParams {
            capping: false,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 8);
        assert_eq!(pd.polys.num_cells(), 6);
    }

    #[test]
    fn cone_center_uses_vtk_transform_path() {
        let pd = cone(&ConeParams {
            center: [1.0, 2.0, 3.0],
            resolution: 3,
            ..Default::default()
        });

        assert_eq!(pd.points.get(0), [1.5, 2.0, 3.0]);
        assert_eq!(pd.points.get(1), [0.5, 1.5, 3.0]);
    }
}
