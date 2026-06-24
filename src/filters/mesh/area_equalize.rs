use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Equalize triangle areas by moving vertices.
///
/// Each vertex moves to reduce area variance among its adjacent triangles.
/// Produces more uniform triangulations.
pub fn equalize_areas(input: &PolyData, iterations: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let tris: Vec<[usize; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Build vertex-to-triangle adjacency
    let mut v_tris: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ti, tri) in tris.iter().enumerate() {
        for &v in tri {
            v_tris[v].push(ti);
        }
    }

    for _ in 0..iterations {
        for i in 0..n {
            if v_tris[i].len() < 2 {
                continue;
            }

            // Compute areas of adjacent triangles
            let areas: Vec<f64> = v_tris[i]
                .iter()
                .map(|&ti| {
                    let t = &tris[ti];
                    tri_area(pts[t[0]], pts[t[1]], pts[t[2]])
                })
                .collect();

            let mean_area: f64 = areas.iter().sum::<f64>() / areas.len() as f64;
            if mean_area < 1e-15 {
                continue;
            }

            // Move toward centroid of large triangles, away from small ones
            let mut dx = 0.0;
            let mut dy = 0.0;
            let mut dz = 0.0;
            for (ai, &ti) in v_tris[i].iter().enumerate() {
                let t = &tris[ti];
                let cx = (pts[t[0]][0] + pts[t[1]][0] + pts[t[2]][0]) / 3.0;
                let cy = (pts[t[0]][1] + pts[t[1]][1] + pts[t[2]][1]) / 3.0;
                let cz = (pts[t[0]][2] + pts[t[1]][2] + pts[t[2]][2]) / 3.0;
                let weight = (areas[ai] - mean_area) / mean_area * 0.1;
                dx += weight * (cx - pts[i][0]);
                dy += weight * (cy - pts[i][1]);
                dz += weight * (cz - pts[i][2]);
            }
            let cnt = v_tris[i].len() as f64;
            pts[i][0] += dx / cnt;
            pts[i][1] += dy / cnt;
            pts[i][2] += dz / cnt;
        }
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Compute area variance of triangle mesh (lower = more uniform).
pub fn area_variance(input: &PolyData) -> f64 {
    let mut areas = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            areas.push(tri_area(
                v0,
                input.points.get(cell[i] as usize),
                input.points.get(cell[i + 1] as usize),
            ));
        }
    }
    if areas.is_empty() {
        return 0.0;
    }
    let mean: f64 = areas.iter().sum::<f64>() / areas.len() as f64;
    areas.iter().map(|a| (a - mean).powi(2)).sum::<f64>() / areas.len() as f64
}

fn tri_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equalize_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 0.1, 0.0]);
        pd.points.push([0.5, 10.0, 0.0]); // very unequal areas
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let before = area_variance(&pd);
        let result = equalize_areas(&pd, 10);
        let after = area_variance(&result);
        assert!(after <= before + 0.01);
    }

    #[test]
    fn variance_uniform() {
        let mut pd = PolyData::new();
        let h = (3.0f64).sqrt() / 2.0;
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.points.push([1.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);

        let v = area_variance(&pd);
        assert!(v < 0.01); // equilateral triangles = same area
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(area_variance(&pd), 0.0);
    }
}
