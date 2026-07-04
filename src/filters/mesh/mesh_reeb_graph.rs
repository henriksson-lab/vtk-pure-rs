//! Reeb graph from a scalar function on a triangle mesh.
use crate::data::PolyData;
use crate::filters::geometry::reeb_graph_filter::poly_data_to_reeb_graph;

pub fn reeb_graph(mesh: &PolyData, scalar_name: &str, n_levels: usize) -> PolyData {
    let _ = n_levels;
    let n_points = mesh.points.len();
    match mesh.point_data().get_array(scalar_name) {
        Some(arr) if arr.num_components() == 1 && arr.num_tuples() == n_points => {}
        _ => return PolyData::new(),
    }
    if mesh.points.len() == 0
        || !mesh.verts.is_empty()
        || !mesh.lines.is_empty()
        || !mesh.strips.is_empty()
        || mesh.polys.iter().any(|cell| {
            cell.len() != 3
                || cell
                    .iter()
                    .any(|&id| usize::try_from(id).map_or(true, |idx| idx >= n_points))
        })
    {
        return PolyData::new();
    }

    poly_data_to_reeb_graph(mesh, scalar_name).to_poly_data()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

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
        assert!(r.points.len() >= 2);
        assert!(r.lines.num_cells() >= 1);
    }
}
