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
        if cell.len() < 3 {
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

    let tris: Vec<[[f64; 3]; 3]> = input
        .polys
        .iter()
        .filter_map(|c| {
            if c.len() >= 3 {
                Some([
                    input.points.get(c[0] as usize),
                    input.points.get(c[1] as usize),
                    input.points.get(c[2] as usize),
                ])
            } else {
                None
            }
        })
        .collect();

    let rays = num_rays.max(1);
    let mut sdf = vec![0.0f64; n];

    for i in 0..n {
        let p = input.points.get(i);
        let nm = vnormals[i];
        let inward = [-nm[0], -nm[1], -nm[2]];

        // Offset slightly to avoid self-intersection
        let origin = [
            p[0] + inward[0] * 0.001,
            p[1] + inward[1] * 0.001,
            p[2] + inward[2] * 0.001,
        ];

        let mut min_t = f64::MAX;
        for tri in &tris {
            if let Some(t) = ray_tri(origin, inward, tri) {
                if t > 0.002 && t < min_t {
                    min_t = t;
                }
            }
        }

        sdf[i] = if min_t < f64::MAX { min_t } else { 0.0 };
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
}
