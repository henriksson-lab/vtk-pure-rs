use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Parameters for generating a rectangular grid of quads.
pub struct GridParams {
    /// Number of points in X direction. Default: 10
    pub x_resolution: usize,
    /// Number of points in Y direction. Default: 10
    pub y_resolution: usize,
    /// Origin corner. Default: [0, 0, 0]
    pub origin: [f64; 3],
    /// X axis vector. Default: [1, 0, 0]
    pub x_axis: [f64; 3],
    /// Y axis vector. Default: [0, 1, 0]
    pub y_axis: [f64; 3],
}

impl Default for GridParams {
    fn default() -> Self {
        Self {
            x_resolution: 10,
            y_resolution: 10,
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
        }
    }
}

/// Generate a rectangular grid of quad cells with texture coordinates and normals.
pub fn grid(params: &GridParams) -> PolyData {
    let nx = params.x_resolution.max(2);
    let ny = params.y_resolution.max(2);

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut tcoords = DataArray::<f64>::new("TCoords", 2);
    let mut polys = CellArray::new();

    // Normal = x_axis × y_axis
    let n = cross(params.x_axis, params.y_axis);
    let nlen = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    let normal = if nlen > 1e-15 {
        [n[0] / nlen, n[1] / nlen, n[2] / nlen]
    } else {
        [0.0, 0.0, 1.0]
    };

    for j in 0..ny {
        let v = j as f64 / (ny - 1) as f64;
        for i in 0..nx {
            let u = i as f64 / (nx - 1) as f64;
            points.push([
                params.origin[0] + u * params.x_axis[0] + v * params.y_axis[0],
                params.origin[1] + u * params.x_axis[1] + v * params.y_axis[1],
                params.origin[2] + u * params.x_axis[2] + v * params.y_axis[2],
            ]);
            normals.push_tuple(&normal);
            tcoords.push_tuple(&[u, v]);
        }
    }

    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            let a = (j * nx + i) as i64;
            let b = a + 1;
            let c = ((j + 1) * nx + i + 1) as i64;
            let d = ((j + 1) * nx + i) as i64;
            polys.push_cell(&[a, b, c, d]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(AnyDataArray::F64(normals));
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut().add_array(AnyDataArray::F64(tcoords));
    pd.point_data_mut().set_active_tcoords("TCoords");
    pd
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_grid() {
        let pd = grid(&GridParams::default());
        assert_eq!(pd.points.len(), 100); // 10x10
        assert_eq!(pd.polys.num_cells(), 81); // 9x9 quads
    }

    #[test]
    fn has_tcoords_and_normals() {
        let pd = grid(&GridParams::default());
        assert!(pd.point_data().get_array("Normals").is_some());
        assert!(pd.point_data().get_array("TCoords").is_some());
    }

    #[test]
    fn small_grid() {
        let pd = grid(&GridParams {
            x_resolution: 3,
            y_resolution: 2,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 6);
        assert_eq!(pd.polys.num_cells(), 2); // 2x1 quads
    }

    #[test]
    fn custom_axes() {
        let pd = grid(&GridParams {
            x_resolution: 2,
            y_resolution: 2,
            origin: [0.0, 0.0, 0.0],
            x_axis: [2.0, 0.0, 0.0],
            y_axis: [0.0, 3.0, 0.0],
        });
        let p = pd.points.get(3); // (1,1) corner
        assert!((p[0] - 2.0).abs() < 1e-10);
        assert!((p[1] - 3.0).abs() < 1e-10);
    }
}
