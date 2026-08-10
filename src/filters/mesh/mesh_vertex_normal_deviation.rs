//! Compute deviation of vertex normals from adjacent face normals.
//!
//! Re-exported from [`crate::filters::mesh::vertex_normal_deviation`], which
//! holds the single implementation. The triangle-strip handling that used to
//! live here has been folded into it.

pub use crate::filters::mesh::vertex_normal_deviation::normal_deviation;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;

    #[test]
    fn test_flat() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = normal_deviation(&m);
        let mut buf = [0.0];
        r.point_data()
            .get_array("NormalDeviation")
            .unwrap()
            .tuple_as_f64(1, &mut buf);
        assert!(buf[0] < 5.0);
    } // flat mesh -> near zero deviation

    #[test]
    fn test_sharp() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = normal_deviation(&m);
        let arr = r.point_data().get_array("NormalDeviation").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 5.0);
    }
}
