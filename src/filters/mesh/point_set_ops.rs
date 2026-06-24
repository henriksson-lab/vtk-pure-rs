//! Point set operations: union, intersection, difference, symmetric diff.

use crate::data::{Points, PolyData};

/// Union of two point clouds (merge, remove duplicates within tolerance).
pub fn point_set_union(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let tol2 = tolerance * tolerance;
    let mut pts = Points::<f64>::new();
    for i in 0..a.points.len() {
        pts.push(a.points.get(i));
    }
    for i in 0..b.points.len() {
        let p = b.points.get(i);
        let dup = (0..pts.len()).any(|j| {
            let q = pts.get(j);
            (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) < tol2
        });
        if !dup {
            pts.push(p);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result
}

/// Intersection: points in A that have a match in B within tolerance.
pub fn point_set_intersection(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let tol2 = tolerance * tolerance;
    let b_pts: Vec<[f64; 3]> = (0..b.points.len()).map(|i| b.points.get(i)).collect();
    let mut pts = Points::<f64>::new();
    for i in 0..a.points.len() {
        let p = a.points.get(i);
        let has_match = b_pts
            .iter()
            .any(|q| (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) < tol2);
        if has_match {
            pts.push(p);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result
}

/// Difference: points in A that have NO match in B.
pub fn point_set_difference(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let tol2 = tolerance * tolerance;
    let b_pts: Vec<[f64; 3]> = (0..b.points.len()).map(|i| b.points.get(i)).collect();
    let mut pts = Points::<f64>::new();
    for i in 0..a.points.len() {
        let p = a.points.get(i);
        let has_match = b_pts
            .iter()
            .any(|q| (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) < tol2);
        if !has_match {
            pts.push(p);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result
}

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
    fn union() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = point_set_union(&a, &b, 0.1);
        assert_eq!(result.points.len(), 3); // [1,0,0] is shared
    }
    #[test]
    fn intersection() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = point_set_intersection(&a, &b, 0.1);
        assert_eq!(result.points.len(), 1); // only [1,0,0]
    }
    #[test]
    fn difference() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0]]);
        let result = point_set_difference(&a, &b, 0.1);
        assert_eq!(result.points.len(), 1); // only [0,0,0]
    }
    #[test]
    fn symmetric() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = point_set_symmetric_difference(&a, &b, 0.1);
        assert_eq!(result.points.len(), 2); // [0,0,0] and [2,0,0]
    }
}
