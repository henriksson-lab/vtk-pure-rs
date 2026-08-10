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

    let n_points = (x_resolution + 1) * (y_resolution + 1);
    let n_quads = x_resolution * y_resolution;
    let mut points = vec![0.0; n_points * 3];
    let mut normals = vec![0.0; n_points * 3];
    let mut tcoords = vec![0.0; n_points * 2];

    for i in 0..=y_resolution {
        let tc1 = i as f64 / y_resolution as f64;
        for j in 0..=x_resolution {
            let tc0 = j as f64 / x_resolution as f64;
            let point_idx = i * (x_resolution + 1) + j;
            let point_base = point_idx * 3;
            points[point_base] = origin[0] + tc0 * v1[0] + tc1 * v2[0];
            points[point_base + 1] = origin[1] + tc0 * v1[1] + tc1 * v2[1];
            points[point_base + 2] = origin[2] + tc0 * v1[2] + tc1 * v2[2];
            normals[point_base] = normal[0];
            normals[point_base + 1] = normal[1];
            normals[point_base + 2] = normal[2];
            let tc_base = point_idx * 2;
            tcoords[tc_base] = tc0;
            tcoords[tc_base + 1] = tc1;
        }
    }

    let mut connectivity = vec![0; n_quads * 4];
    for i in 0..y_resolution {
        for j in 0..x_resolution {
            let pt0 = (j + i * (x_resolution + 1)) as i64;
            let pt1 = pt0 + 1;
            let pt2 = pt0 + x_resolution as i64 + 2;
            let pt3 = pt0 + x_resolution as i64 + 1;
            let base = (i * x_resolution + j) * 4;
            connectivity[base] = pt0;
            connectivity[base + 1] = pt1;
            connectivity[base + 2] = pt2;
            connectivity[base + 3] = pt3;
        }
    }
    let offsets: Vec<i64> = (0..=n_quads).map(|i| (i * 4) as i64).collect();

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(points);
    pd.polys = CellArray::from_raw(offsets, connectivity);
    pd.point_data_mut()
        .add_array(DataArray::from_vec("Normals", normals, 3).into());
    pd.point_data_mut().set_active_normals("Normals");
    pd.point_data_mut()
        .add_array(DataArray::from_vec("TextureCoordinates", tcoords, 2).into());
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
