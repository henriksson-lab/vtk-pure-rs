use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute visibility from a viewpoint: fraction of hemisphere not occluded.
///
/// For each vertex, casts rays in a hemisphere around its normal and counts
/// how many are blocked by other triangles. Adds "Visibility" scalar (0-1).
/// Simplified version using a small number of sample directions.
pub fn ambient_occlusion(input: &PolyData, num_rays: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if !valid_cell(cell, n) {
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

    // Collect triangles for ray testing
    let tris = collect_triangles(input);

    let rays = num_rays.max(4);
    let mut visibility = vec![0.0f64; n];

    for i in 0..n {
        let p = input.points.get(i);
        let nm = vnormals[i];
        let mut unblocked = 0;

        // Generate hemisphere directions around normal
        for ri in 0..rays {
            let phi = 2.0 * std::f64::consts::PI * ri as f64 / rays as f64;
            let theta = std::f64::consts::PI * 0.25; // 45 degrees from normal

            // Local frame
            let up = if nm[2].abs() < 0.9 {
                [0.0, 0.0, 1.0]
            } else {
                [1.0, 0.0, 0.0]
            };
            let t = normalize(cross(nm, up));
            let b = cross(nm, t);

            let st = theta.sin();
            let ct = theta.cos();
            let dir = [
                ct * nm[0] + st * (phi.cos() * t[0] + phi.sin() * b[0]),
                ct * nm[1] + st * (phi.cos() * t[1] + phi.sin() * b[1]),
                ct * nm[2] + st * (phi.cos() * t[2] + phi.sin() * b[2]),
            ];

            // Test ray against triangles
            let offset = [
                p[0] + dir[0] * 0.001,
                p[1] + dir[1] * 0.001,
                p[2] + dir[2] * 0.001,
            ];
            let mut hit = false;
            for tri in &tris {
                if tri.contains(&i) {
                    continue;
                }
                let tri_pts = [
                    input.points.get(tri[0]),
                    input.points.get(tri[1]),
                    input.points.get(tri[2]),
                ];
                if ray_tri(offset, dir, &tri_pts) {
                    hit = true;
                    break;
                }
            }
            if !hit {
                unblocked += 1;
            }
        }

        visibility[i] = unblocked as f64 / rays as f64;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Visibility",
            visibility,
            1,
        )));
    pd
}

fn ray_tri(o: [f64; 3], d: [f64; 3], tri: &[[f64; 3]; 3]) -> bool {
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
    let h = cross(d, e2);
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return false;
    }
    let f = 1.0 / a;
    let s = [o[0] - tri[0][0], o[1] - tri[0][1], o[2] - tri[0][2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let q = cross(s, e1);
    let v = f * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]) > 0.001
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 1e-15 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [1.0, 0.0, 0.0]
    }
}

fn collect_triangles(input: &PolyData) -> Vec<[usize; 3]> {
    let mut triangles = Vec::new();
    for cell in input.polys.iter() {
        if !valid_cell(cell, input.points.len()) {
            continue;
        }
        for i in 1..cell.len() - 1 {
            triangles.push([cell[0] as usize, cell[i] as usize, cell[i + 1] as usize]);
        }
    }
    triangles
}

fn valid_cell(cell: &[i64], num_points: usize) -> bool {
    cell.len() >= 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_triangle_full_visibility() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = ambient_occlusion(&pd, 8);
        let arr = result.point_data().get_array("Visibility").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.5); // mostly unoccluded
    }

    #[test]
    fn has_visibility_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = ambient_occlusion(&pd, 4);
        assert!(result.point_data().get_array("Visibility").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = ambient_occlusion(&pd, 8);
        assert_eq!(result.points.len(), 0);
    }
}
