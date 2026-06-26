use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute Shape Diameter Function (SDF) at each vertex.
///
/// For each vertex, casts rays inward (opposite to normal) and measures
/// distance to opposite surface. Approximates local thickness.
/// Adds "ShapeDiameter" scalar.
pub fn shape_diameter_function(input: &PolyData, num_rays: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if !valid_cell(input, cell) {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &id in cell.iter() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
        }
    }
    for nm in &mut vnormals {
        let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if l > 1e-15 {
            nm[0] /= l;
            nm[1] /= l;
            nm[2] /= l;
        }
    }

    let mut tris: Vec<[[f64; 3]; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if !valid_cell(input, cell) {
            continue;
        }
        let p0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            tris.push([
                p0,
                input.points.get(cell[i] as usize),
                input.points.get(cell[i + 1] as usize),
            ]);
        }
    }

    let rays = num_rays.max(1);
    let mut sdf = vec![0.0f64; n];

    for i in 0..n {
        let p = input.points.get(i);
        let nm = vnormals[i];
        let inward = [-nm[0], -nm[1], -nm[2]];
        let ray_dirs = ray_directions(inward, rays);

        // Offset slightly to avoid self-intersection
        let origin = [
            p[0] + inward[0] * 0.001,
            p[1] + inward[1] * 0.001,
            p[2] + inward[2] * 0.001,
        ];

        let mut sum_t = 0.0;
        let mut hit_count = 0usize;
        for dir in ray_dirs {
            let mut min_t = f64::MAX;
            for tri in &tris {
                if let Some(t) = ray_tri(origin, dir, tri) {
                    if t > 0.002 && t < min_t {
                        min_t = t;
                    }
                }
            }
            if min_t < f64::MAX {
                sum_t += min_t;
                hit_count += 1;
            }
        }

        sdf[i] = if hit_count > 0 {
            sum_t / hit_count as f64
        } else {
            0.0
        };
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ShapeDiameter",
            sdf,
            1,
        )));
    pd
}

fn valid_cell(input: &PolyData, cell: &[i64]) -> bool {
    cell.len() >= 3
        && cell
            .iter()
            .all(|&id| id >= 0 && (id as usize) < input.points.len())
}

fn ray_directions(axis: [f64; 3], rays: usize) -> Vec<[f64; 3]> {
    if rays <= 1 {
        return vec![axis];
    }

    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len <= 1e-15 {
        return vec![axis; rays];
    }

    let w = [axis[0] / len, axis[1] / len, axis[2] / len];
    let helper = if w[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let u0 = [
        helper[1] * w[2] - helper[2] * w[1],
        helper[2] * w[0] - helper[0] * w[2],
        helper[0] * w[1] - helper[1] * w[0],
    ];
    let u_len = (u0[0] * u0[0] + u0[1] * u0[1] + u0[2] * u0[2]).sqrt();
    let u = [u0[0] / u_len, u0[1] / u_len, u0[2] / u_len];
    let v = [
        w[1] * u[2] - w[2] * u[1],
        w[2] * u[0] - w[0] * u[2],
        w[0] * u[1] - w[1] * u[0],
    ];

    let cone_sin = 30.0_f64.to_radians().sin();
    let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    let mut dirs = Vec::with_capacity(rays);
    dirs.push(w);
    for k in 1..rays {
        let frac = (k as f64 / (rays - 1).max(1) as f64).sqrt();
        let radial = cone_sin * frac;
        let axial = (1.0 - radial * radial).sqrt();
        let theta = k as f64 * golden_angle;
        let (s, c) = theta.sin_cos();
        dirs.push([
            w[0] * axial + (u[0] * c + v[0] * s) * radial,
            w[1] * axial + (u[1] * c + v[1] * s) * radial,
            w[2] * axial + (u[2] * c + v[2] * s) * radial,
        ]);
    }
    dirs
}

fn ray_tri(o: [f64; 3], d: [f64; 3], tri: &[[f64; 3]; 3]) -> Option<f64> {
    let e1 = [
        tri[1][0] - tri[0][0],
        tri[1][1] - tri[0][1],
        tri[1][2] - tri[0][2],
    ];
    let e2 = [
        tri[2][0] - tri[0][0],
        tri[2][1] - tri[0][1],
        tri[2][2] - tri[0][2],
    ];
    let h = [
        d[1] * e2[2] - d[2] * e2[1],
        d[2] * e2[0] - d[0] * e2[2],
        d[0] * e2[1] - d[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = [o[0] - tri[0][0], o[1] - tri[0][1], o[2] - tri[0][2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    Some(f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdf_basic() {
        let mut pd = PolyData::new();
        // Box-like: two parallel triangles
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 2.0]);
        pd.points.push([1.0, 0.0, 2.0]);
        pd.points.push([0.5, 1.0, 2.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[5, 4, 3]); // opposite winding

        let result = shape_diameter_function(&pd, 1);
        assert!(result.point_data().get_array("ShapeDiameter").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = shape_diameter_function(&pd, 4);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn skips_invalid_polygons() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 99]);
        pd.polys.push_cell(&[0, -1, 2]);

        let result = shape_diameter_function(&pd, 2);
        assert_eq!(result.points.len(), 3);
        assert!(result.point_data().get_array("ShapeDiameter").is_some());
    }
}
