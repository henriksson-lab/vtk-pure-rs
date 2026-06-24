use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute deviation of vertex normals from face normals.
///
/// For each vertex, measures how much its smooth normal deviates from
/// the adjacent face normals. High deviation = sharp feature.
/// Adds "NormalDeviation" scalar (angle in degrees).
pub fn normal_deviation(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute vertex normals (area-weighted average)
    let mut vnormals = vec![[0.0f64; 3]; n];
    let mut face_normals_per_vertex: Vec<Vec<[f64; 3]>> = vec![Vec::new(); n];

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
        let fl = (fn_[0] * fn_[0] + fn_[1] * fn_[1] + fn_[2] * fn_[2]).sqrt();
        let fn_n = if fl > 1e-15 {
            [fn_[0] / fl, fn_[1] / fl, fn_[2] / fl]
        } else {
            [0.0; 3]
        };

        for &id in cell.iter() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
            face_normals_per_vertex[i].push(fn_n);
        }
    }

    // Normalize vertex normals
    for nm in &mut vnormals {
        let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if l > 1e-15 {
            nm[0] /= l;
            nm[1] /= l;
            nm[2] /= l;
        }
    }

    // Compute max deviation angle for each vertex
    let mut deviation = vec![0.0f64; n];
    for i in 0..n {
        let vn = vnormals[i];
        let mut max_angle = 0.0f64;
        for fn_ in &face_normals_per_vertex[i] {
            let dot = (vn[0] * fn_[0] + vn[1] * fn_[1] + vn[2] * fn_[2]).clamp(-1.0, 1.0);
            let angle = dot.acos().to_degrees();
            max_angle = max_angle.max(angle);
        }
        deviation[i] = max_angle;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalDeviation",
            deviation,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_zero_deviation() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = normal_deviation(&pd);
        let arr = result.point_data().get_array("NormalDeviation").unwrap();
        let mut buf = [0.0f64];
        // Flat surface: all normals agree -> low deviation
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] < 5.0);
        }
    }

    #[test]
    fn sharp_high_deviation() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = normal_deviation(&pd);
        let arr = result.point_data().get_array("NormalDeviation").unwrap();
        let mut buf = [0.0f64];
        // Vertices 0 and 1 are on the sharp edge -> high deviation
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 10.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = normal_deviation(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
