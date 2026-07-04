//! Resample points uniformly on mesh surface using face area weighting.
use crate::data::{CellArray, Points, PolyData};

pub fn resample_uniform(mesh: &PolyData, target_count: usize, seed: u64) -> PolyData {
    let mut triangles = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || !valid_cell_points(cell, mesh.points.len()) {
            continue;
        }
        for i in 1..cell.len() - 1 {
            let tri = [cell[0], cell[i], cell[i + 1]];
            let area = triangle_area(mesh, tri);
            if area > 0.0 {
                triangles.push((tri, area));
            }
        }
    }
    if triangles.is_empty() {
        return PolyData::new();
    }
    let total: f64 = triangles.iter().map(|(_, area)| *area).sum();
    if total < 1e-30 {
        return PolyData::new();
    }
    let mut cum = Vec::with_capacity(triangles.len());
    let mut acc = 0.0;
    for &(_, area) in &triangles {
        acc += area / total;
        cum.push(acc);
    }
    let mut rng = seed;
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for _ in 0..target_count {
        let r = next_random(&mut rng);
        let ci = cum.partition_point(|&c| c < r).min(triangles.len() - 1);
        let mut u = next_random(&mut rng);
        let mut v = next_random(&mut rng);
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let w = 1.0 - u - v;
        let tri = triangles[ci].0;
        let a = mesh.points.get(tri[0] as usize);
        let b = mesh.points.get(tri[1] as usize);
        let c = mesh.points.get(tri[2] as usize);
        let idx = pts.len();
        pts.push([
            a[0] * w + b[0] * u + c[0] * v,
            a[1] * w + b[1] * u + c[1] * v,
            a[2] * w + b[2] * u + c[2] * v,
        ]);
        verts.push_cell(&[idx as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}

fn next_random(rng: &mut u64) -> f64 {
    *rng = rng
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*rng >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
}

fn valid_cell_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&id| usize::try_from(id).ok().is_some_and(|idx| idx < num_points))
}

fn triangle_area(mesh: &PolyData, tri: [i64; 3]) -> f64 {
    let a = mesh.points.get(tri[0] as usize);
    let b = mesh.points.get(tri[1] as usize);
    let c = mesh.points.get(tri[2] as usize);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
    .sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = resample_uniform(&m, 50, 42);
        assert_eq!(r.points.len(), 50);
    }
}
