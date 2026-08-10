//! Point set operations: union, intersection, difference, symmetric diff.

use crate::data::PolyData;

pub use crate::filters::mesh::boolean_point_set::{
    point_set_difference, point_set_intersection, point_set_union,
};

/// Symmetric difference: points in A not in B, plus points in B not in A.
pub fn point_set_symmetric_difference(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let d1 = point_set_difference(a, b, tolerance);
    let d2 = point_set_difference(b, a, tolerance);
    point_set_union(&d1, &d2, tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn symmetric() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = point_set_symmetric_difference(&a, &b, 0.1);
        assert_eq!(result.points.len(), 2); // [0,0,0] and [2,0,0]
    }
}
