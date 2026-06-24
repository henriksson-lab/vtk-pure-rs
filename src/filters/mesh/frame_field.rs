use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute a tangent frame field on a triangle mesh.
///
/// For each vertex, computes a local orthonormal frame (tangent, bitangent, normal)
/// from the vertex normal and the first edge direction. Adds "Tangent" and
/// "Bitangent" 3-component arrays.
pub fn compute_frame_field(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    let mut first_edge = vec![[0.0f64; 3]; n];
    let mut has_edge = vec![false; n];

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
        for (idx, &id) in cell.iter().enumerate() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
            if !has_edge[i] {
                let next = cell[(idx + 1) % cell.len()] as usize;
                let p = input.points.get(i);
                let q = input.points.get(next);
                first_edge[i] = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                has_edge[i] = true;
            }
        }
    }

    let mut tangents = vec![[0.0f64; 3]; n];
    let mut bitangents = vec![[0.0f64; 3]; n];

    for i in 0..n {
        let nm = &mut vnormals[i];
        let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if l > 1e-15 {
            nm[0] /= l;
            nm[1] /= l;
            nm[2] /= l;
        }

        // Project edge onto tangent plane
        let e = first_edge[i];
        let dot = e[0] * nm[0] + e[1] * nm[1] + e[2] * nm[2];
        let t = [e[0] - dot * nm[0], e[1] - dot * nm[1], e[2] - dot * nm[2]];
        let tl = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
        if tl > 1e-15 {
            tangents[i] = [t[0] / tl, t[1] / tl, t[2] / tl];
            let b = [
                nm[1] * tangents[i][2] - nm[2] * tangents[i][1],
                nm[2] * tangents[i][0] - nm[0] * tangents[i][2],
                nm[0] * tangents[i][1] - nm[1] * tangents[i][0],
            ];
            bitangents[i] = b;
        }
    }

    let t_flat: Vec<f64> = tangents.iter().flat_map(|t| t.iter().copied()).collect();
    let b_flat: Vec<f64> = bitangents.iter().flat_map(|b| b.iter().copied()).collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Tangent", t_flat, 3)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Bitangent",
            b_flat,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_on_flat() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = compute_frame_field(&pd);
        assert!(result.point_data().get_array("Tangent").is_some());
        assert!(result.point_data().get_array("Bitangent").is_some());

        let t = result.point_data().get_array("Tangent").unwrap();
        let mut buf = [0.0f64; 3];
        t.tuple_as_f64(0, &mut buf);
        // Tangent should be in XY plane
        assert!(buf[2].abs() < 1e-10);
    }

    #[test]
    fn orthogonal_frame() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = compute_frame_field(&pd);
        let ta = result.point_data().get_array("Tangent").unwrap();
        let ba = result.point_data().get_array("Bitangent").unwrap();
        let mut tb = [0.0f64; 3];
        let mut bb = [0.0f64; 3];
        ta.tuple_as_f64(0, &mut tb);
        ba.tuple_as_f64(0, &mut bb);
        let dot = tb[0] * bb[0] + tb[1] * bb[1] + tb[2] * bb[2];
        assert!(dot.abs() < 1e-10); // orthogonal
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = compute_frame_field(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
