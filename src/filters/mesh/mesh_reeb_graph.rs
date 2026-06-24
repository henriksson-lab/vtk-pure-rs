//! Simplified Reeb graph from a scalar function on a mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn reeb_graph(mesh: &PolyData, scalar_name: &str, n_levels: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let arr = match mesh.point_data().get_array(scalar_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let mut vals: Vec<f64> = Vec::with_capacity(n);
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        vals.push(buf[0]);
    }
    let vmin = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let vmax = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (vmax - vmin).abs() < 1e-15 {
        return mesh.clone();
    }

    let levels = n_levels.max(2);
    // Assign each vertex to a level
    let level_of: Vec<usize> = vals
        .iter()
        .map(|&v| {
            let t = (v - vmin) / (vmax - vmin);
            (t * (levels - 1) as f64).round() as usize
        })
        .collect();

    let mut result = mesh.clone();
    let level_data: Vec<f64> = level_of.iter().map(|&l| l as f64).collect();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ReebLevel",
            level_data,
            1,
        )));
    result.point_data_mut().set_active_scalars("ReebLevel");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_reeb() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let heights: Vec<f64> = (0..4).map(|i| mesh.points.get(i)[1]).collect();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("height", heights, 1)));
        let r = reeb_graph(&mesh, "height", 5);
        assert!(r.point_data().get_array("ReebLevel").is_some());
    }
}
