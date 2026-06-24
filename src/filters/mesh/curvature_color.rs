use crate::data::{AnyDataArray, DataArray, PolyData};

/// Color vertices by curvature using a diverging colormap.
///
/// Computes mean curvature and maps it to RGB: blue=convex, red=concave,
/// white=flat. Adds "CurvatureColor" 3-component array.
pub fn curvature_color(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    // Compute signed curvature (Laplacian dot normal)
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

    let mut curvatures = vec![0.0f64; n];
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let p = input.points.get(i);
        let nm = vnormals[i];
        let cnt = neighbors[i].len() as f64;
        let mut lx = 0.0;
        let mut ly = 0.0;
        let mut lz = 0.0;
        for &j in &neighbors[i] {
            let q = input.points.get(j);
            lx += q[0] - p[0];
            ly += q[1] - p[1];
            lz += q[2] - p[2];
        }
        lx /= cnt;
        ly /= cnt;
        lz /= cnt;
        curvatures[i] = lx * nm[0] + ly * nm[1] + lz * nm[2]; // signed curvature
    }

    // Normalize to [-1,1]
    let max_abs = curvatures
        .iter()
        .map(|c| c.abs())
        .fold(0.0f64, f64::max)
        .max(1e-15);

    let mut colors = Vec::with_capacity(n * 3);
    for i in 0..n {
        let t = (curvatures[i] / max_abs).clamp(-1.0, 1.0);
        if t > 0.0 {
            // Concave -> red
            colors.push(1.0);
            colors.push(1.0 - t);
            colors.push(1.0 - t);
        } else {
            // Convex -> blue
            let s = -t;
            colors.push(1.0 - s);
            colors.push(1.0 - s);
            colors.push(1.0);
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureColor",
            colors,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_color_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = curvature_color(&pd);
        let arr = result.point_data().get_array("CurvatureColor").unwrap();
        assert_eq!(arr.num_components(), 3);
    }

    #[test]
    fn flat_white() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let result = curvature_color(&pd);
        let arr = result.point_data().get_array("CurvatureColor").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(4, &mut buf); // center vertex: should be near white
                                       // All components should be near 1.0 for flat region
        assert!(buf[0] > 0.5 && buf[1] > 0.5 && buf[2] > 0.5);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = curvature_color(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
