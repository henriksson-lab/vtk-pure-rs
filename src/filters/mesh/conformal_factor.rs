use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashSet;

/// Compute the conformal factor (area distortion) relative to a reference mesh.
///
/// For each triangle, computes log(area_current / area_reference).
/// Zero = no area change, positive = expansion, negative = compression.
/// Adds "ConformalFactor" cell data.
pub fn conformal_factor(current: &PolyData, reference: &PolyData) -> PolyData {
    let nc = current.polys.num_cells();
    let nr = reference.polys.num_cells();
    if nc == 0 || nr == 0 || nc != nr {
        return current.clone();
    }

    let mut factors = Vec::with_capacity(nc);
    let mut ci = current.polys.iter();
    let mut ri = reference.polys.iter();

    loop {
        let cc = ci.next();
        let rc = ri.next();
        match (cc, rc) {
            (Some(c), Some(r)) => {
                if c.len() < 3 || r.len() < 3 {
                    factors.push(0.0);
                    continue;
                }
                let ca = tri_area_cell(current, c);
                let ra = tri_area_cell(reference, r);
                factors.push(if ra > 1e-15 { (ca / ra).ln() } else { 0.0 });
            }
            _ => break,
        }
    }

    let mut pd = current.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ConformalFactor",
            factors,
            1,
        )));
    pd
}

/// Compute angle distortion between current and reference meshes.
///
/// For each triangle, computes max angle difference between corresponding
/// triangles. Adds "AngleDistortion" cell data (in degrees).
pub fn angle_distortion(current: &PolyData, reference: &PolyData) -> PolyData {
    let nc = current.polys.num_cells();
    let nr = reference.polys.num_cells();
    if nc == 0 || nr == 0 || nc != nr {
        return current.clone();
    }

    let mut distortion = Vec::with_capacity(nc);
    let mut ci = current.polys.iter();
    let mut ri = reference.polys.iter();

    loop {
        let cc = ci.next();
        let rc = ri.next();
        match (cc, rc) {
            (Some(c), Some(r)) if c.len() >= 3 && r.len() >= 3 => {
                let ca = tri_angles(current, c);
                let ra = tri_angles(reference, r);
                let max_diff = (ca[0] - ra[0])
                    .abs()
                    .max((ca[1] - ra[1]).abs())
                    .max((ca[2] - ra[2]).abs());
                distortion.push(max_diff.to_degrees());
            }
            (Some(_), Some(_)) => distortion.push(0.0),
            _ => break,
        }
    }

    let mut pd = current.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AngleDistortion",
            distortion,
            1,
        )));
    pd
}

fn tri_area_cell(pd: &PolyData, c: &[i64]) -> f64 {
    let v0 = pd.points.get(c[0] as usize);
    let v1 = pd.points.get(c[1] as usize);
    let v2 = pd.points.get(c[2] as usize);
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

fn tri_angles(pd: &PolyData, c: &[i64]) -> [f64; 3] {
    let v = [
        pd.points.get(c[0] as usize),
        pd.points.get(c[1] as usize),
        pd.points.get(c[2] as usize),
    ];
    let mut angles = [0.0; 3];
    for i in 0..3 {
        let e1 = [
            v[(i + 1) % 3][0] - v[i][0],
            v[(i + 1) % 3][1] - v[i][1],
            v[(i + 1) % 3][2] - v[i][2],
        ];
        let e2 = [
            v[(i + 2) % 3][0] - v[i][0],
            v[(i + 2) % 3][1] - v[i][1],
            v[(i + 2) % 3][2] - v[i][2],
        ];
        let dot = e1[0] * e2[0] + e1[1] * e2[1] + e1[2] * e2[2];
        let l1 = (e1[0] * e1[0] + e1[1] * e1[1] + e1[2] * e1[2]).sqrt();
        let l2 = (e2[0] * e2[0] + e2[1] * e2[1] + e2[2] * e2[2]).sqrt();
        angles[i] = if l1 > 1e-15 && l2 > 1e-15 {
            (dot / (l1 * l2)).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        };
    }
    angles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_mesh_zero_factor() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = conformal_factor(&pd, &pd);
        let arr = result.cell_data().get_array("ConformalFactor").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0].abs() < 1e-10); // no distortion
    }

    #[test]
    fn angle_distortion_same() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = angle_distortion(&pd, &pd);
        let arr = result.cell_data().get_array("AngleDistortion").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 0.01); // same mesh = no distortion
    }

    #[test]
    fn scaled_mesh_factor() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([0.0, 1.0, 0.0]);
        a.polys.push_cell(&[0, 1, 2]);

        let mut b = PolyData::new();
        b.points.push([0.0, 0.0, 0.0]);
        b.points.push([2.0, 0.0, 0.0]);
        b.points.push([0.0, 2.0, 0.0]);
        b.polys.push_cell(&[0, 1, 2]);

        let result = conformal_factor(&b, &a);
        let arr = result.cell_data().get_array("ConformalFactor").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 1.0); // b is 4x area -> ln(4) ≈ 1.39
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = conformal_factor(&pd, &pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
