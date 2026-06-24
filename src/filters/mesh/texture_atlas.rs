use crate::data::{AnyDataArray, DataArray, PolyData};

/// Simple per-triangle texture atlas parameterization.
///
/// Assigns each triangle its own rectangular UV patch in a grid layout.
/// Useful for baking procedural textures or lightmaps. Adds "AtlasUV"
/// 2-component array. Vertices are duplicated so each triangle has unique UVs.
pub fn texture_atlas(input: &PolyData) -> PolyData {
    let tris: Vec<Vec<i64>> = input
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .map(|c| c.to_vec())
        .collect();
    let n_tris = tris.len();
    if n_tris == 0 {
        return input.clone();
    }

    // Grid layout: ceil(sqrt(n_tris)) x ceil(sqrt(n_tris))
    let grid = (n_tris as f64).sqrt().ceil() as usize;
    let cell_size = 1.0 / grid as f64;

    let mut out_points = crate::data::Points::<f64>::new();
    let mut out_polys = crate::data::CellArray::new();
    let mut uvs = Vec::new();

    for (ti, tri) in tris.iter().enumerate() {
        let row = ti / grid;
        let col = ti % grid;
        let u_base = col as f64 * cell_size;
        let v_base = row as f64 * cell_size;

        let base = out_points.len() as i64;
        let mut ids = Vec::new();

        // Map triangle vertices to UV positions within their cell
        let uv_coords = [[0.1, 0.1], [0.9, 0.1], [0.5, 0.9]];
        for (vi, &pid) in tri.iter().enumerate().take(3) {
            out_points.push(input.points.get(pid as usize));
            let uv = if vi < 3 { uv_coords[vi] } else { [0.5, 0.5] };
            uvs.push(u_base + uv[0] * cell_size);
            uvs.push(v_base + uv[1] * cell_size);
            ids.push(base + vi as i64);
        }
        out_polys.push_cell(&ids);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("AtlasUV", uvs, 2)));
    pd.point_data_mut().set_active_tcoords("AtlasUV");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = texture_atlas(&pd);
        assert_eq!(result.points.len(), 6); // 2 * 3
        assert!(result.point_data().get_array("AtlasUV").is_some());
        let arr = result.point_data().get_array("AtlasUV").unwrap();
        assert_eq!(arr.num_components(), 2);
    }

    #[test]
    fn uvs_in_range() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = texture_atlas(&pd);
        let arr = result.point_data().get_array("AtlasUV").unwrap();
        let mut buf = [0.0f64; 2];
        for i in 0..result.points.len() {
            arr.tuple_as_f64(i, &mut buf);
            assert!(buf[0] >= 0.0 && buf[0] <= 1.0);
            assert!(buf[1] >= 0.0 && buf[1] <= 1.0);
        }
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = texture_atlas(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
