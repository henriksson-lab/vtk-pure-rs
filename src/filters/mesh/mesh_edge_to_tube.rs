//! Convert mesh edges to tube geometry (for visualization).
use crate::data::{CellArray, Points, PolyData};
pub fn edges_to_tubes(mesh: &PolyData, radius: f64, sides: usize) -> PolyData {
    let sides = sides.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in mesh.lines.iter() {
        for pair in cell.windows(2) {
            add_tube(
                mesh, radius, sides, pair[0], pair[1], &mut seen, &mut pts, &mut polys,
            );
        }
    }
    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            add_tube(
                mesh,
                radius,
                sides,
                cell[i],
                cell[(i + 1) % cell.len()],
                &mut seen,
                &mut pts,
                &mut polys,
            );
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            add_tube(
                mesh, radius, sides, tri[0], tri[1], &mut seen, &mut pts, &mut polys,
            );
            add_tube(
                mesh, radius, sides, tri[1], tri[2], &mut seen, &mut pts, &mut polys,
            );
            add_tube(
                mesh, radius, sides, tri[2], tri[0], &mut seen, &mut pts, &mut polys,
            );
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
fn add_tube(
    mesh: &PolyData,
    radius: f64,
    sides: usize,
    a: i64,
    b: i64,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    pts: &mut Points<f64>,
    polys: &mut CellArray,
) {
    let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) else {
        return;
    };
    if a >= mesh.points.len() || b >= mesh.points.len() || a == b {
        return;
    }
    let key = (a.min(b), a.max(b));
    if !seen.insert(key) {
        return;
    }
    let pa = mesh.points.get(a);
    let pb = mesh.points.get(b);
    let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if dl < 1e-15 {
        return;
    }
    let t = [d[0] / dl, d[1] / dl, d[2] / dl];
    let up = if t[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let n1 = normalize(cross(t, up));
    let n2 = cross(t, n1);
    let base = pts.len();
    for ring in 0..2 {
        let center = if ring == 0 { pa } else { pb };
        for s in 0..sides {
            let ang = 2.0 * std::f64::consts::PI * s as f64 / sides as f64;
            pts.push([
                center[0] + radius * (ang.cos() * n1[0] + ang.sin() * n2[0]),
                center[1] + radius * (ang.cos() * n1[1] + ang.sin() * n2[1]),
                center[2] + radius * (ang.cos() * n1[2] + ang.sin() * n2[2]),
            ]);
        }
    }
    for s in 0..sides {
        let s1 = (s + 1) % sides;
        polys.push_cell(&[
            (base + s) as i64,
            (base + s1) as i64,
            (base + sides + s1) as i64,
            (base + sides + s) as i64,
        ]);
    }
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = edges_to_tubes(&m, 0.05, 6);
        assert_eq!(r.points.len(), 36);
        assert_eq!(r.polys.num_cells(), 18);
    }
}
