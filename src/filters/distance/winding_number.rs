use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the generalized winding number for probe points relative to
/// a closed triangle surface.
///
/// The winding number is approximately 1.0 for points inside the surface,
/// 0.0 for points outside, and 0.5 on the surface. More robust than
/// ray-casting for non-watertight meshes.
///
/// Adds a "WindingNumber" scalar to the probe's point data.
pub fn winding_number(surface: &PolyData, probe: &PolyData) -> PolyData {
    let tris: Vec<[[f64; 3]; 3]> = surface
        .polys
        .iter()
        .flat_map(|cell| {
            let v0 = surface.points.get(cell[0] as usize);
            (1..cell.len() - 1).map(move |i| {
                [
                    v0,
                    surface.points.get(cell[i] as usize),
                    surface.points.get(cell[i + 1] as usize),
                ]
            })
        })
        .collect();

    let n = probe.points.len();
    let mut wn = vec![0.0f64; n];

    for (pi, w) in wn.iter_mut().enumerate() {
        let p = probe.points.get(pi);
        let mut sum = 0.0;
        for tri in &tris {
            sum += solid_angle(p, tri);
        }
        *w = sum / (4.0 * std::f64::consts::PI);
    }

    let mut pd = probe.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WindingNumber",
            wn,
            1,
        )));
    pd
}

/// Solid angle subtended by a triangle as seen from a point.
/// Uses the Van Oosterom-Strackee formula.
fn solid_angle(p: [f64; 3], tri: &[[f64; 3]; 3]) -> f64 {
    let a = sub(tri[0], p);
    let b = sub(tri[1], p);
    let c = sub(tri[2], p);

    let la = length(a);
    let lb = length(b);
    let lc = length(c);

    if la < 1e-15 || lb < 1e-15 || lc < 1e-15 {
        return 0.0;
    }

    let numerator = dot(a, cross(b, c));
    let denominator = la * lb * lc + dot(a, b) * lc + dot(a, c) * lb + dot(b, c) * la;

    2.0 * numerator.atan2(denominator)
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn length(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box() -> PolyData {
        let mut pd = PolyData::new();
        let c = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        for p in &c {
            pd.points.push(*p);
        }
        let faces = [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [2, 3, 7, 6],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
        ];
        for f in &faces {
            pd.polys.push_cell(&[f[0] as i64, f[1] as i64, f[2] as i64]);
            pd.polys.push_cell(&[f[0] as i64, f[2] as i64, f[3] as i64]);
        }
        pd
    }

    #[test]
    fn inside_point() {
        let surface = make_box();
        let mut probe = PolyData::new();
        probe.points.push([0.5, 0.5, 0.5]);

        let result = winding_number(&surface, &probe);
        let arr = result.point_data().get_array("WindingNumber").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        // Should be ~1.0 for point inside
        assert!(buf[0].abs() > 0.8, "winding number = {}", buf[0]);
    }

    #[test]
    fn outside_point() {
        let surface = make_box();
        let mut probe = PolyData::new();
        probe.points.push([5.0, 5.0, 5.0]);

        let result = winding_number(&surface, &probe);
        let arr = result.point_data().get_array("WindingNumber").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        // Should be ~0.0 for point outside
        assert!(buf[0].abs() < 0.2, "winding number = {}", buf[0]);
    }

    #[test]
    fn multiple_probe_points() {
        let surface = make_box();
        let mut probe = PolyData::new();
        probe.points.push([0.5, 0.5, 0.5]); // inside
        probe.points.push([5.0, 5.0, 5.0]); // outside
        probe.points.push([0.2, 0.2, 0.2]); // inside

        let result = winding_number(&surface, &probe);
        let arr = result.point_data().get_array("WindingNumber").unwrap();
        assert_eq!(arr.num_tuples(), 3);
    }
}
