use crate::data::{DataSet, PolyData};

/// Result of a point picking operation.
#[derive(Debug, Clone)]
pub struct PickResult {
    pub point_id: usize,
    pub position: [f64; 3],
    pub distance: f64,
}

/// Find the closest mesh point to a 3D query position.
pub fn pick_closest_point(input: &PolyData, query: [f64; 3]) -> Option<PickResult> {
    let n = input.points.len();
    if n == 0 {
        return None;
    }

    let mut best_id = 0;
    let mut best_d2 = f64::MAX;
    for i in 0..n {
        let p = input.points.get(i);
        let d2 = (p[0] - query[0]).powi(2) + (p[1] - query[1]).powi(2) + (p[2] - query[2]).powi(2);
        if d2 < best_d2 {
            best_d2 = d2;
            best_id = i;
        }
    }

    Some(PickResult {
        point_id: best_id,
        position: input.points.get(best_id),
        distance: best_d2.sqrt(),
    })
}

/// Find all points within a sphere of given radius around query.
pub fn pick_points_in_sphere(input: &PolyData, center: [f64; 3], radius: f64) -> Vec<PickResult> {
    let r2 = radius * radius;
    let mut results = Vec::new();
    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let d2 =
            (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2);
        if d2 <= r2 {
            results.push(PickResult {
                point_id: i,
                position: p,
                distance: d2.sqrt(),
            });
        }
    }
    results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    results
}

/// Find the closest cell to a query point (by cell centroid distance).
pub fn pick_closest_cell(input: &PolyData, query: [f64; 3]) -> Option<usize> {
    let mut best_id = None;
    let mut best_d2 = f64::MAX;
    for (ci, cell) in input.polys.iter().enumerate() {
        if cell.is_empty() {
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &id in cell.iter() {
            let p = input.points.get(id as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let n = cell.len() as f64;
        let d2 = ((cx / n - query[0]).powi(2)
            + (cy / n - query[1]).powi(2)
            + (cz / n - query[2]).powi(2));
        if d2 < best_d2 {
            best_d2 = d2;
            best_id = Some(ci);
        }
    }
    best_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);

        let result = pick_closest_point(&pd, [0.9, 0.0, 0.0]).unwrap();
        assert_eq!(result.point_id, 1);
        assert!((result.distance - 0.1).abs() < 1e-10);
    }

    #[test]
    fn pick_in_sphere() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);

        let results = pick_points_in_sphere(&pd, [0.5, 0.0, 0.0], 1.0);
        assert_eq!(results.len(), 2); // points 0 and 1
    }

    #[test]
    fn pick_cell() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([5.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let ci = pick_closest_cell(&pd, [5.5, 0.3, 0.0]).unwrap();
        assert_eq!(ci, 1); // second triangle
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert!(pick_closest_point(&pd, [0.0; 3]).is_none());
    }
}
