use std::f64::consts::PI;

use crate::data::{CellArray, DataArray, Points, PolyData};

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
            resolution: 16,
            capping: true,
        }
    }
}

/// Generate a cone as PolyData.
///
/// The cone axis runs along `direction` with the apex at `center + direction * height/2`
/// and the base at `center - direction * height/2`.
pub fn cone(params: &ConeParams) -> PolyData {
    let n = params.resolution.max(3);
    let [cx, cy, cz] = params.center;
    let h = params.height;
    let r = params.radius;

    // Normalize direction
    let d = params.direction;
    let dlen = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let dir = if dlen > 1e-10 {
        [d[0] / dlen, d[1] / dlen, d[2] / dlen]
    } else {
        [1.0, 0.0, 0.0]
    };

    // Find two perpendicular vectors to dir
    let (u, v) = perpendicular_frame(dir);

    // Apex point
    let apex = [
        cx + dir[0] * h * 0.5,
        cy + dir[1] * h * 0.5,
        cz + dir[2] * h * 0.5,
    ];
    // Base center
    let base_center = [
        cx - dir[0] * h * 0.5,
        cy - dir[1] * h * 0.5,
        cz - dir[2] * h * 0.5,
    ];

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // Point 0: apex
    points.push(apex);
    normals.push_tuple(&dir);

    // Base ring points (1..=n)
    for i in 0..n {
        let theta = 2.0 * PI * i as f64 / n as f64;
        let ct = theta.cos();
        let st = theta.sin();

        let px = base_center[0] + r * (ct * u[0] + st * v[0]);
        let py = base_center[1] + r * (ct * u[1] + st * v[1]);
        let pz = base_center[2] + r * (ct * u[2] + st * v[2]);

        // Normal: outward from the cone surface
        let radial = [
            ct * u[0] + st * v[0],
            ct * u[1] + st * v[1],
            ct * u[2] + st * v[2],
        ];
        // Cone surface normal is a blend of radial and axis direction
        let slope = r / h;
        let nx = radial[0] + dir[0] * slope;
        let ny = radial[1] + dir[1] * slope;
        let nz = radial[2] + dir[2] * slope;
        let nlen = (nx * nx + ny * ny + nz * nz).sqrt();

        points.push([px, py, pz]);
        normals.push_tuple(&[nx / nlen, ny / nlen, nz / nlen]);
    }

    // Side triangles
    for i in 0..n {
        let next = (i + 1) % n;
        polys.push_cell(&[0, (1 + i) as i64, (1 + next) as i64]);
    }

    // Base cap
    if params.capping {
        let base_idx = points.len() as i64;
        points.push(base_center);
        normals.push_tuple(&[-dir[0], -dir[1], -dir[2]]);

        // Duplicate base ring for cap normals
        for i in 0..n {
            let theta = 2.0 * PI * i as f64 / n as f64;
            let ct = theta.cos();
            let st = theta.sin();
            let px = base_center[0] + r * (ct * u[0] + st * v[0]);
            let py = base_center[1] + r * (ct * u[1] + st * v[1]);
            let pz = base_center[2] + r * (ct * u[2] + st * v[2]);
            points.push([px, py, pz]);
            normals.push_tuple(&[-dir[0], -dir[1], -dir[2]]);
        }

        for i in 0..n {
            let next = (i + 1) % n;
            polys.push_cell(&[
                base_idx,
                base_idx + 1 + next as i64,
                base_idx + 1 + i as i64,
            ]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

fn perpendicular_frame(dir: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    // Find a vector not parallel to dir
    let seed = if dir[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };

    // u = normalize(seed x dir)
    let u = cross(seed, dir);
    let ulen = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    let u = [u[0] / ulen, u[1] / ulen, u[2] / ulen];

    // v = dir x u
    let v = cross(dir, u);
    (u, v)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
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
        // Without cap: apex + ring = 1 + resolution
        assert_eq!(pd.points.len(), 1 + 16);
        assert_eq!(pd.polys.num_cells(), 16); // side triangles only
    }
}
