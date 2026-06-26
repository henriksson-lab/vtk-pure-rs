use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a rectangular plane.
pub struct PlaneParams {
    pub origin: [f64; 3],
    pub point1: [f64; 3],
    pub point2: [f64; 3],
    pub x_resolution: usize,
    pub y_resolution: usize,
}

impl Default for PlaneParams {
    fn default() -> Self {
        Self {
            origin: [-0.5, -0.5, 0.0],
            point1: [0.5, -0.5, 0.0],
            point2: [-0.5, 0.5, 0.0],
            x_resolution: 1,
            y_resolution: 1,
        }
    }
}

/// Generate a rectangular plane as PolyData with normals and texture coordinates.
pub fn plane(params: &PlaneParams) -> PolyData {
    let x_resolution = params.x_resolution.max(1);
    let y_resolution = params.y_resolution.max(1);

    let origin = params.origin;
    let point1 = params.point1;
    let point2 = params.point2;
    let v1 = [
        point1[0] - origin[0],
        point1[1] - origin[1],
        point1[2] - origin[2],
    ];
    let v2 = [
        point2[0] - origin[0],
        point2[1] - origin[1],
        point2[2] - origin[2],
    ];

    let mut normal = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    let normal_length =
        (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if normal_length == 0.0 {
        return PolyData::new();
    }
    for value in &mut normal {
        *value /= normal_length;
    }

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut tcoords = DataArray::<f64>::new("TextureCoordinates", 2);
    let mut polys = CellArray::new();

    for i in 0..=y_resolution {
        let tc1 = i as f64 / y_resolution as f64;
        for j in 0..=x_resolution {
            let tc0 = j as f64 / x_resolution as f64;
            points.push([
                origin[0] + tc0 * v1[0] + tc1 * v2[0],
                origin[1] + tc0 * v1[1] + tc1 * v2[1],
                origin[2] + tc0 * v1[2] + tc1 * v2[2],
            ]);
            normals.push_tuple(&normal);
            tcoords.push_tuple(&[tc0, tc1]);
        }
    }

    for i in 0..y_resolution {
        for j in 0..x_resolution {
            let pt0 = (j + i * (x_resolution + 1)) as i64;
            let pt1 = pt0 + 1;
            let pt2 = pt0 + x_resolution as i64 + 2;
            let pt3 = pt0 + x_resolution as i64 + 1;
            polys.push_cell(&[pt0, pt1, pt2, pt3]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut().add_array(tcoords.into());
    pd.point_data_mut().set_active_tcoords("TextureCoordinates");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_plane() {
        let pd = plane(&PlaneParams::default());
        assert_eq!(pd.points.len(), 4); // 2x2 grid
        assert_eq!(pd.polys.num_cells(), 1); // 1 quad
    }

    #[test]
    fn subdivided_plane() {
        let pd = plane(&PlaneParams {
            x_resolution: 3,
            y_resolution: 2,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 4 * 3); // 4 cols * 3 rows
        assert_eq!(pd.polys.num_cells(), 3 * 2); // 3x2 quads
    }

    #[test]
    fn degenerate_plane_is_empty() {
        let pd = plane(&PlaneParams {
            point1: [-0.5, 0.5, 0.0],
            point2: [-0.5, 0.5, 0.0],
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 0);
        assert_eq!(pd.polys.num_cells(), 0);
    }
}
