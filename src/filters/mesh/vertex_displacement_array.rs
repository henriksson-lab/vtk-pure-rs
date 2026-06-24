use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-vertex displacement between two meshes.
///
/// Both meshes must have the same number of points. Adds two arrays to
/// point data of a clone of `source`:
/// - "Displacement" (3-component): the vector from source to target per vertex
/// - "DisplacementMagnitude" (1-component): the length of each displacement vector
pub fn compute_displacement(source: &PolyData, target: &PolyData) -> PolyData {
    let n: usize = source.points.len();
    assert_eq!(
        n,
        target.points.len(),
        "Source and target must have the same number of points"
    );

    let mut displacements: Vec<f64> = Vec::with_capacity(n * 3);
    let mut magnitudes: Vec<f64> = Vec::with_capacity(n);

    for i in 0..n {
        let s = source.points.get(i);
        let t = target.points.get(i);
        let dx: f64 = t[0] - s[0];
        let dy: f64 = t[1] - s[1];
        let dz: f64 = t[2] - s[2];
        displacements.push(dx);
        displacements.push(dy);
        displacements.push(dz);
        let mag: f64 = (dx * dx + dy * dy + dz * dz).sqrt();
        magnitudes.push(mag);
    }

    let mut pd = source.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Displacement",
            displacements,
            3,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DisplacementMagnitude",
            magnitudes,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_displacement() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_displacement(&pd, &pd);
        let mag = result
            .point_data()
            .get_array("DisplacementMagnitude")
            .unwrap();
        let mut val = [0.0f64];
        for i in 0..3 {
            mag.tuple_as_f64(i, &mut val);
            assert!(val[0].abs() < 1e-12);
        }
    }

    #[test]
    fn unit_displacement() {
        let source = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Shift all points by (1, 0, 0)
        let target = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_displacement(&source, &target);

        let disp = result.point_data().get_array("Displacement").unwrap();
        let mut vec3 = [0.0f64; 3];
        disp.tuple_as_f64(0, &mut vec3);
        assert!((vec3[0] - 1.0).abs() < 1e-12);
        assert!(vec3[1].abs() < 1e-12);
        assert!(vec3[2].abs() < 1e-12);

        let mag = result
            .point_data()
            .get_array("DisplacementMagnitude")
            .unwrap();
        let mut val = [0.0f64];
        mag.tuple_as_f64(0, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn diagonal_displacement() {
        let source = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Shift point 0 by (1, 1, 1)
        let target = PolyData::from_triangles(
            vec![[1.0, 1.0, 1.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_displacement(&source, &target);
        let mag = result
            .point_data()
            .get_array("DisplacementMagnitude")
            .unwrap();
        let mut val = [0.0f64];
        mag.tuple_as_f64(0, &mut val);
        let expected: f64 = 3.0f64.sqrt();
        assert!((val[0] - expected).abs() < 1e-12);
    }
}
