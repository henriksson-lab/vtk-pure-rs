use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute tangent vectors for a triangle mesh.
///
/// Uses Lengyel's method: for each triangle, computes the tangent
/// from texture coordinate gradients. If no texture coordinates exist,
/// computes tangent from the first edge direction. Adds a "Tangents"
/// 3-component point data array.
pub fn compute_tangents(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut tangents = vec![[0.0f64; 3]; n];

    let has_tcoords = input.point_data().get_array("TCoords").is_some();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);

        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        let t = if has_tcoords {
            let tc = input.point_data().get_array("TCoords").unwrap();
            let mut uv0 = [0.0f64; 2];
            tc.tuple_as_f64(cell[0] as usize, &mut uv0);
            let mut uv1 = [0.0f64; 2];
            tc.tuple_as_f64(cell[1] as usize, &mut uv1);
            let mut uv2 = [0.0f64; 2];
            tc.tuple_as_f64(cell[2] as usize, &mut uv2);

            let du1 = uv1[0] - uv0[0];
            let dv1 = uv1[1] - uv0[1];
            let du2 = uv2[0] - uv0[0];
            let dv2 = uv2[1] - uv0[1];
            let det = du1 * dv2 - du2 * dv1;

            if det.abs() > 1e-15 {
                let inv = 1.0 / det;
                [
                    inv * (dv2 * e1[0] - dv1 * e2[0]),
                    inv * (dv2 * e1[1] - dv1 * e2[1]),
                    inv * (dv2 * e1[2] - dv1 * e2[2]),
                ]
            } else {
                e1
            }
        } else {
            e1
        };

        for &id in cell.iter() {
            let idx = id as usize;
            tangents[idx][0] += t[0];
            tangents[idx][1] += t[1];
            tangents[idx][2] += t[2];
        }
    }

    // Normalize
    for t in &mut tangents {
        let len = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        if len > 1e-15 {
            t[0] /= len;
            t[1] /= len;
            t[2] /= len;
        }
    }

    let flat: Vec<f64> = tangents.iter().flat_map(|t| t.iter().copied()).collect();
    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Tangents", flat, 3)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tangents() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = compute_tangents(&pd);
        assert!(result.point_data().get_array("Tangents").is_some());
        let arr = result.point_data().get_array("Tangents").unwrap();
        assert_eq!(arr.num_components(), 3);
    }

    #[test]
    fn tangent_direction() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = compute_tangents(&pd);
        let arr = result.point_data().get_array("Tangents").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        // Tangent should be along edge 0->1 = +X
        assert!(buf[0] > 0.5);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = compute_tangents(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
