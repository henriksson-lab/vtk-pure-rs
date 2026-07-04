use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute mean curvature at each vertex using the discrete Laplace-Beltrami.
///
/// H = ||Δp|| / 2, where Δp is the uniform Laplacian. More accurate than
/// the angle-deficit Gaussian curvature. Adds "MeanCurvature" scalar.
pub fn discrete_mean_curvature(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut curv = vec![0.0f64; n];
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let p = input.points.get(i);
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
        curv[i] = (lx * lx + ly * ly + lz * lz).sqrt() * 0.5;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MeanCurvature",
            curv,
            1,
        )));
    pd
}

/// Classify each vertex as convex, concave, or saddle based on curvature sign.
/// Adds "CurvatureType" scalar: 1=convex, -1=concave, 0=flat.
pub fn curvature_classification(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if cell.iter().any(|&id| id < 0 || id as usize >= n) {
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

    let mut types = vec![0.0f64; n];
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
        let dot = lx * nm[0] + ly * nm[1] + lz * nm[2];
        types[i] = if dot > 0.001 {
            -1.0
        } else if dot < -0.001 {
            1.0
        } else {
            0.0
        };
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurvatureType",
            types,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_zero_curvature() {
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

        let result = discrete_mean_curvature(&pd);
        let arr = result.point_data().get_array("MeanCurvature").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf); // center
        assert!(buf[0] < 0.1);
    }

    #[test]
    fn classification_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[2, 0, 3]);

        let result = curvature_classification(&pd);
        assert!(result.point_data().get_array("CurvatureType").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = discrete_mean_curvature(&pd);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn skips_cells_with_invalid_point_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 99]);

        let mean = discrete_mean_curvature(&pd);
        assert_eq!(
            mean.point_data()
                .get_array("MeanCurvature")
                .unwrap()
                .num_tuples(),
            2
        );

        let classed = curvature_classification(&pd);
        assert_eq!(
            classed
                .point_data()
                .get_array("CurvatureType")
                .unwrap()
                .num_tuples(),
            2
        );
    }
}
