use crate::data::{Points, PolyData};

/// Tangential smoothing: smooth only in the tangent plane.
///
/// Moves vertices along the surface without changing the overall shape.
/// Each vertex moves toward the average of its neighbors, but the
/// component along the normal is removed.
pub fn tangential_smooth(input: &PolyData, lambda: f64, iterations: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for _ in 0..iterations {
        let mut vnormals = vec![[0.0f64; 3]; n];
        for cell in input.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let v0 = pts[cell[0] as usize];
            let v1 = pts[cell[1] as usize];
            let v2 = pts[cell[2] as usize];
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let fn_ = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            for &id in cell.iter() {
                let i = id as usize;
                vnormals[i][0] += fn_[0];
                vnormals[i][1] += fn_[1];
                vnormals[i][2] += fn_[2];
            }
        }
        for nm in &mut vnormals {
            let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
            if l > 1e-15 {
                nm[0] /= l;
                nm[1] /= l;
                nm[2] /= l;
            }
        }

        let mut new_pts = pts.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let p = pts[i];
            let nm = vnormals[i];
            let cnt = neighbors[i].len() as f64;
            let mut dx = 0.0;
            let mut dy = 0.0;
            let mut dz = 0.0;
            for &j in &neighbors[i] {
                dx += pts[j][0] - p[0];
                dy += pts[j][1] - p[1];
                dz += pts[j][2] - p[2];
            }
            dx /= cnt;
            dy /= cnt;
            dz /= cnt;

            // Remove normal component
            let ndot = dx * nm[0] + dy * nm[1] + dz * nm[2];
            dx -= ndot * nm[0];
            dy -= ndot * nm[1];
            dz -= ndot * nm[2];

            new_pts[i] = [p[0] + lambda * dx, p[1] + lambda * dy, p[2] + lambda * dz];
        }
        pts = new_pts;
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_plane() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let result = tangential_smooth(&pd, 0.5, 5);
        // Z should remain 0 (tangential only)
        for i in 0..9 {
            let p = result.points.get(i);
            assert!(p[2].abs() < 1e-10);
        }
    }

    #[test]
    fn zero_lambda() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.points.push([7.0, 8.0, 9.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        let result = tangential_smooth(&pd, 0.0, 10);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(tangential_smooth(&pd, 0.5, 5).points.len(), 0);
    }
}
