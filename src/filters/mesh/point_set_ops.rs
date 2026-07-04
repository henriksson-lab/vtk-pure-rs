//! Point set operations: union, intersection, difference, symmetric diff.

use crate::data::{CellArray, Points, PolyData};

/// Union of two point clouds (merge, remove duplicates within tolerance).
pub fn point_set_union(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let tol2 = tolerance * tolerance;
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..a.points.len() {
        append_unique_vertex(&mut pts, &mut verts, a.points.get(i), tol2);
    }
    for i in 0..b.points.len() {
        append_unique_vertex(&mut pts, &mut verts, b.points.get(i), tol2);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

/// Intersection: points in A that have a match in B within tolerance.
pub fn point_set_intersection(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let tol2 = tolerance * tolerance;
    let b_pts: Vec<[f64; 3]> = (0..b.points.len()).map(|i| b.points.get(i)).collect();
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..a.points.len() {
        let p = a.points.get(i);
        let has_match = b_pts.iter().any(|q| squared_distance(p, *q) <= tol2);
        if has_match {
            append_unique_vertex(&mut pts, &mut verts, p, tol2);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

/// Difference: points in A that have NO match in B.
pub fn point_set_difference(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let tol2 = tolerance * tolerance;
    let b_pts: Vec<[f64; 3]> = (0..b.points.len()).map(|i| b.points.get(i)).collect();
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..a.points.len() {
        let p = a.points.get(i);
        let has_match = b_pts.iter().any(|q| squared_distance(p, *q) <= tol2);
        if !has_match {
            append_unique_vertex(&mut pts, &mut verts, p, tol2);
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.verts = verts;
    result
}

/// Symmetric difference: points in A not in B, plus points in B not in A.
pub fn point_set_symmetric_difference(a: &PolyData, b: &PolyData, tolerance: f64) -> PolyData {
    let d1 = point_set_difference(a, b, tolerance);
    let d2 = point_set_difference(b, a, tolerance);
    point_set_union(&d1, &d2, tolerance)
}

fn append_unique_vertex(
    points: &mut Points<f64>,
    verts: &mut CellArray,
    point: [f64; 3],
    tolerance_squared: f64,
) {
    if (0..points.len()).any(|i| squared_distance(points.get(i), point) <= tolerance_squared) {
        return;
    }

    let idx = points.len() as i64;
    points.push(point);
    verts.push_cell(&[idx]);
}

fn squared_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
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
        assert_eq!(result.verts.num_cells(), 3);
    }
    #[test]
    fn union_deduplicates_first_input() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.05, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0]]);
        let result = point_set_union(&a, &b, 0.1);
        assert_eq!(result.points.len(), 2);
        assert_eq!(result.verts.num_cells(), 2);
    }
    #[test]
    fn intersection() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = point_set_intersection(&a, &b, 0.1);
        assert_eq!(result.points.len(), 1); // only [1,0,0]
        assert_eq!(result.verts.num_cells(), 1);
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
