//! Thicken a surface mesh into a solid shell.
//!
//! The single implementation lives in [`crate::filters::mesh::thicken`]; this
//! module only re-exports it so the historical path keeps working.

pub use crate::filters::mesh::thicken::thicken;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, PolyData};

    #[test]
    fn test_thicken_duplicates_point_data() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "ids",
                vec![3, 5, 7],
                1,
            )));
        let r = thicken(&m, 0.5);
        match r.point_data().get_array("ids").unwrap() {
            AnyDataArray::I32(array) => {
                assert_eq!(array.num_tuples(), 6);
                assert_eq!(array.tuple(0), &[3]);
                assert_eq!(array.tuple(3), &[3]);
            }
            other => panic!("unexpected array type: {:?}", other.scalar_type()),
        }
    }
}
