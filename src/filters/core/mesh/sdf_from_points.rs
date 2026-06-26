use crate::data::{AnyDataArray, DataArray, ImageData, KdTree, PolyData};

/// Create an approximate signed distance field from a point cloud with normals.
///
/// For each grid point, finds the nearest input point and uses the
/// dot product with the normal to determine the sign. Adds "SDF" scalar.
pub fn sdf_from_oriented_points(
    input: &PolyData,
    dimensions: [usize; 3],
    padding: f64,
) -> ImageData {
    use crate::data::DataSet;

    let n = input.points.len();
    if n == 0 {
        return ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
    }

    let normals_arr = input.point_data().get_array("Normals");
    let has_normals = normals_arr
        .map(|a| a.num_components() == 3)
        .unwrap_or(false);

    let bb = input.bounds();
    let origin = [bb.x_min - padding, bb.y_min - padding, bb.z_min - padding];
    let sp = [
        (bb.x_max - bb.x_min + 2.0 * padding) / (dimensions[0] as f64 - 1.0).max(1.0),
        (bb.y_max - bb.y_min + 2.0 * padding) / (dimensions[1] as f64 - 1.0).max(1.0),
        (bb.z_max - bb.z_min + 2.0 * padding) / (dimensions[2] as f64 - 1.0).max(1.0),
    ];

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let nx = dimensions[0];
    let ny = dimensions[1];
    let nz = dimensions[2];
    let mut sdf = Vec::with_capacity(nx * ny * nz);
    let mut nbuf = [0.0f64; 3];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [
                    origin[0] + i as f64 * sp[0],
                    origin[1] + j as f64 * sp[1],
                    origin[2] + k as f64 * sp[2],
                ];
                if let Some((idx, d2)) = tree.nearest(p) {
                    let d = d2.sqrt();
                    let sign = if has_normals {
                        let narr = normals_arr.unwrap();
                        narr.tuple_as_f64(idx, &mut nbuf);
                        let dp = (p[0] - pts[idx][0]) * nbuf[0]
                            + (p[1] - pts[idx][1]) * nbuf[1]
                            + (p[2] - pts[idx][2]) * nbuf[2];
                        if dp >= 0.0 {
                            1.0
                        } else {
                            -1.0
                        }
                    } else {
                        1.0
                    };
                    sdf.push(sign * d);
                } else {
                    sdf.push(f64::MAX);
                }
            }
        }
    }

    let mut img = ImageData::with_dimensions(nx, ny, nz);
    img.set_origin(origin);
    img.set_spacing(sp);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("SDF", sdf, 1)));
    img.point_data_mut().set_active_scalars("SDF");
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdf_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        let img = sdf_from_oriented_points(&pd, [5, 5, 1], 0.5);
        assert_eq!(img.dimensions(), [5, 5, 1]);
        assert!(img.point_data().get_array("SDF").is_some());
    }

    #[test]
    fn sdf_with_normals() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0],
                3,
            )));

        let img = sdf_from_oriented_points(&pd, [3, 3, 3], 1.0);
        let arr = img.point_data().get_array("SDF").unwrap();
        assert_eq!(arr.num_tuples(), 27);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let img = sdf_from_oriented_points(&pd, [3, 3, 3], 1.0);
        assert_eq!(img.dimensions(), [3, 3, 3]);
    }
}
