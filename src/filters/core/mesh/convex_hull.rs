//! 2D and 3D convex hull computation.

use crate::data::{CellArray, Points, PolyData};

/// Compute 2D convex hull (in XY plane) using Graham scan.
pub fn convex_hull_2d(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n < 3 { return mesh.clone(); }

    let mut points: Vec<(usize, [f64; 3])> = (0..n).map(|i| (i, mesh.points.get(i))).collect();
    // Find bottom-left point
    points.sort_by(|a, b| {
        a.1[1].partial_cmp(&b.1[1]).unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1[0].partial_cmp(&b.1[0]).unwrap_or(std::cmp::Ordering::Equal))
    });
    let pivot = points[0].1;

    // Sort by angle
    points[1..].sort_by(|a, b| {
        let aa = (a.1[1] - pivot[1]).atan2(a.1[0] - pivot[0]);
        let ab = (b.1[1] - pivot[1]).atan2(b.1[0] - pivot[0]);
        aa.partial_cmp(&ab)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                let da = (a.1[0] - pivot[0]).powi(2) + (a.1[1] - pivot[1]).powi(2);
                let db = (b.1[0] - pivot[0]).powi(2) + (b.1[1] - pivot[1]).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let mut hull: Vec<usize> = Vec::new();
    for &(idx, p) in &points {
        while hull.len() >= 2 {
            let a = mesh.points.get(hull[hull.len() - 2]);
            let b = mesh.points.get(hull[hull.len() - 1]);
            let cross = (b[0]-a[0])*(p[1]-a[1]) - (b[1]-a[1])*(p[0]-a[0]);
            if cross <= 0.0 { hull.pop(); } else { break; }
        }
        hull.push(idx);
    }

    let mut pts = Points::<f64>::new();
    let ids: Vec<i64> = hull.iter().enumerate().map(|(i, &v)| {
        pts.push(mesh.points.get(v));
        i as i64
    }).collect();
    let mut polys = CellArray::new();
    polys.push_cell(&ids);
    let mut result = PolyData::new();
    result.points = pts; result.polys = polys; result
}

/// Extract convex hull as wireframe (lines).
pub fn convex_hull_2d_wireframe(mesh: &PolyData) -> PolyData {
    let hull = convex_hull_2d(mesh);
    if hull.polys.num_cells() == 0 { return hull; }
    let cell: Vec<i64> = hull.polys.iter().next().unwrap().to_vec();
    let n = cell.len();
    let mut lines = CellArray::new();
    for i in 0..n {
        lines.push_cell(&[cell[i], cell[(i + 1) % n]]);
    }
    let mut result = PolyData::new();
    result.points = hull.points;
    result.lines = lines;
    result
}

/// Compute convex hull area (2D, XY plane).
pub fn convex_hull_area(mesh: &PolyData) -> f64 {
    let hull = convex_hull_2d(mesh);
    if hull.polys.num_cells() == 0 { return 0.0; }
    let cell: Vec<i64> = hull.polys.iter().next().unwrap().to_vec();
    let n = cell.len();
    let mut area = 0.0;
    for i in 0..n {
        let a = hull.points.get(cell[i] as usize);
        let b = hull.points.get(cell[(i + 1) % n] as usize);
        area += a[0] * b[1] - b[0] * a[1];
    }
    area.abs() * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_square() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([0.5, 0.5, 0.0]); // interior point
        let hull = convex_hull_2d(&mesh);
        let cell: Vec<i64> = hull.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 4); // interior point excluded
    }
    #[test]
    fn test_area() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([2.0, 0.0, 0.0]);
        mesh.points.push([2.0, 3.0, 0.0]);
        mesh.points.push([0.0, 3.0, 0.0]);
        let a = convex_hull_area(&mesh);
        assert!((a - 6.0).abs() < 1e-10);
    }
}
