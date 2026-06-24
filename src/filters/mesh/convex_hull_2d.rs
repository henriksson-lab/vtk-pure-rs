use crate::data::{CellArray, Points, PolyData};

/// Compute the 2D convex hull of a point set projected to XY.
///
/// Returns a PolyData with a single polygon cell representing the hull.
/// Uses Andrew's monotone chain algorithm.
pub fn convex_hull_2d(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n < 3 {
        return PolyData::new();
    }

    let mut pts: Vec<(f64, f64, usize)> = (0..n)
        .map(|i| {
            let p = input.points.get(i);
            (p[0], p[1], i)
        })
        .collect();
    pts.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then(a.1.partial_cmp(&b.1).unwrap())
    });

    let cross = |o: (f64, f64, usize), a: (f64, f64, usize), b: (f64, f64, usize)| -> f64 {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    };

    // Lower hull
    let mut lower: Vec<(f64, f64, usize)> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }

    // Upper hull
    let mut upper: Vec<(f64, f64, usize)> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);

    let mut out_points = Points::<f64>::new();
    let mut ids = Vec::with_capacity(lower.len());
    for (x, y, _) in &lower {
        let idx = out_points.len() as i64;
        out_points.push([*x, *y, 0.0]);
        ids.push(idx);
    }

    let mut out_polys = CellArray::new();
    if ids.len() >= 3 {
        out_polys.push_cell(&ids);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Check if a point is inside the 2D convex hull.
pub fn point_in_convex_hull_2d(hull: &PolyData, px: f64, py: f64) -> bool {
    for cell in hull.polys.iter() {
        let n = cell.len();
        for i in 0..n {
            let a = hull.points.get(cell[i] as usize);
            let b = hull.points.get(cell[(i + 1) % n] as usize);
            let cross = (b[0] - a[0]) * (py - a[1]) - (b[1] - a[1]) * (px - a[0]);
            if cross < -1e-10 {
                return false;
            }
        }
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_hull() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 0.0]); // interior point

        let hull = convex_hull_2d(&pd);
        assert_eq!(hull.polys.num_cells(), 1);
        let cell: Vec<i64> = hull.polys.iter().next().unwrap().to_vec();
        assert_eq!(cell.len(), 4); // 4 hull vertices
    }

    #[test]
    fn collinear_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        let hull = convex_hull_2d(&pd);
        assert!(hull.polys.num_cells() >= 1);
    }

    #[test]
    fn point_inside() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        let hull = convex_hull_2d(&pd);

        assert!(point_in_convex_hull_2d(&hull, 1.0, 1.0));
        assert!(!point_in_convex_hull_2d(&hull, 5.0, 5.0));
    }

    #[test]
    fn too_few_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let hull = convex_hull_2d(&pd);
        assert_eq!(hull.polys.num_cells(), 0);
    }
}
