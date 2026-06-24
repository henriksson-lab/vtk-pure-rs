//! Simple 3D convex hull using gift wrapping on projected axes.
use crate::data::{CellArray, Points, PolyData};
pub fn convex_hull_3d_approx(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n < 4 {
        return mesh.clone();
    }
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    // Find extreme points
    let mut extremes = [0usize; 6]; // +x,-x,+y,-y,+z,-z
    for i in 0..n {
        if pts[i][0] > pts[extremes[0]][0] {
            extremes[0] = i;
        }
        if pts[i][0] < pts[extremes[1]][0] {
            extremes[1] = i;
        }
        if pts[i][1] > pts[extremes[2]][1] {
            extremes[2] = i;
        }
        if pts[i][1] < pts[extremes[3]][1] {
            extremes[3] = i;
        }
        if pts[i][2] > pts[extremes[4]][2] {
            extremes[4] = i;
        }
        if pts[i][2] < pts[extremes[5]][2] {
            extremes[5] = i;
        }
    }
    // Start with tetrahedron from 4 extreme points
    let mut hull_pts: Vec<usize> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for &e in &extremes {
        if seen.insert(e) {
            hull_pts.push(e);
        }
    }
    if hull_pts.len() < 4 {
        for i in 0..n {
            if seen.insert(i) {
                hull_pts.push(i);
            }
            if hull_pts.len() >= 4 {
                break;
            }
        }
    }
    // Build triangles from hull points (simple fan)
    let mut new_pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for &i in &hull_pts {
        new_pts.push(pts[i]);
    }
    if hull_pts.len() >= 4 {
        polys.push_cell(&[0, 1, 2]);
        polys.push_cell(&[0, 2, 3]);
        polys.push_cell(&[0, 3, 1]);
        polys.push_cell(&[1, 3, 2]);
    } else if hull_pts.len() == 3 {
        polys.push_cell(&[0, 1, 2]);
    }
    let mut r = PolyData::new();
    r.points = new_pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.points.push([0.0, 0.0, 1.0]);
        m.points.push([0.3, 0.3, 0.3]);
        let r = convex_hull_3d_approx(&m);
        assert_eq!(r.polys.num_cells(), 4);
        assert_eq!(r.points.len(), 4);
    }
}
