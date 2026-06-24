//! Select mesh elements by spatial regions: sphere, box, plane half-space.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Select points inside a sphere.
pub fn select_points_in_sphere(mesh: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    let r2 = radius * radius;
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        if (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2)
            <= r2
        {
            pts.push(p);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result
}

/// Select points inside an axis-aligned box.
pub fn select_points_in_box(mesh: &PolyData, min: [f64; 3], max: [f64; 3]) -> PolyData {
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        let p = mesh.points.get(i);
        if p[0] >= min[0]
            && p[0] <= max[0]
            && p[1] >= min[1]
            && p[1] <= max[1]
            && p[2] >= min[2]
            && p[2] <= max[2]
        {
            pts.push(p);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result
}

/// Select cells whose centroids are in a half-space (positive side of plane).
pub fn select_cells_in_halfspace(
    mesh: &PolyData,
    plane_point: [f64; 3],
    plane_normal: [f64; 3],
) -> PolyData {
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for cell in &all_cells {
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &pid in cell {
            let p = mesh.points.get(pid as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let k = cell.len() as f64;
        let dot = (cx / k - plane_point[0]) * plane_normal[0]
            + (cy / k - plane_point[1]) * plane_normal[1]
            + (cz / k - plane_point[2]) * plane_normal[2];
        if dot >= 0.0 {
            let mut ids = Vec::new();
            for &pid in cell {
                let old = pid as usize;
                let idx = *pt_map.entry(old).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(old));
                    i
                });
                ids.push(idx as i64);
            }
            polys.push_cell(&ids);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Mark points inside a sphere with a scalar array.
pub fn mark_points_in_sphere(mesh: &PolyData, center: [f64; 3], radius: f64) -> PolyData {
    let r2 = radius * radius;
    let data: Vec<f64> = (0..mesh.points.len())
        .map(|i| {
            let p = mesh.points.get(i);
            if (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2)
                <= r2
            {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("InSphere", data, 1)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_select() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [5.0, 0.0, 0.0]]);
        let result = select_points_in_sphere(&mesh, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(result.points.len(), 2);
    }
    #[test]
    fn box_select() {
        let mesh = PolyData::from_points(vec![[0.5, 0.5, 0.5], [2.0, 0.0, 0.0]]);
        let result = select_points_in_box(&mesh, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(result.points.len(), 1);
    }
    #[test]
    fn halfspace() {
        let mesh = PolyData::from_triangles(
            vec![
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-2.0, 0.0, 0.0],
                [-1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = select_cells_in_halfspace(&mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }
    #[test]
    fn mark() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]]);
        let result = mark_points_in_sphere(&mesh, [0.0, 0.0, 0.0], 1.0);
        let arr = result.point_data().get_array("InSphere").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
