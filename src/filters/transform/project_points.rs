use crate::data::{Points, PolyData};

/// Project each point onto the nearest location on a target surface.
///
/// For each point in `input`, finds the closest point on any triangle
/// in `surface` and moves the point there.
pub fn project_to_surface(input: &PolyData, surface: &PolyData) -> PolyData {
    let tris: Vec<([f64; 3], [f64; 3], [f64; 3])> = surface
        .polys
        .iter()
        .flat_map(|cell| {
            let p0 = surface.points.get(cell[0] as usize);
            (1..cell.len() - 1).map(move |i| {
                (
                    p0,
                    surface.points.get(cell[i] as usize),
                    surface.points.get(cell[i + 1] as usize),
                )
            })
        })
        .collect();

    let mut out_points = Points::<f64>::new();

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let mut best_proj = p;
        let mut best_d2 = f64::MAX;

        for &(a, b, c) in &tris {
            let (proj, d2) = closest_point_on_triangle(p, a, b, c);
            if d2 < best_d2 {
                best_d2 = d2;
                best_proj = proj;
            }
        }
        out_points.push(best_proj);
    }

    let mut pd = input.clone();
    pd.points = out_points;
    pd
}

fn closest_point_on_triangle(
    p: [f64; 3],
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
) -> ([f64; 3], f64) {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, dist2(p, a));
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (b, dist2(p, b));
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (c, dist2(p, c));
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let proj = [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
        return (proj, dist2(p, proj));
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let proj = [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
        return (proj, dist2(p, proj));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let proj = [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
        return (proj, dist2(p, proj));
    }
    let d = 1.0 / (va + vb + vc);
    let v = vb * d;
    let w = vc * d;
    let proj = [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ];
    (proj, dist2(p, proj))
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = sub(a, b);
    dot(d, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_onto_plane() {
        let surface = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut input = PolyData::new();
        input.points.push([1.0, 1.0, 5.0]); // 5 units above plane

        let result = project_to_surface(&input, &surface);
        let p = result.points.get(0);
        assert!((p[2]).abs() < 1e-10); // projected onto z=0
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn project_onto_edge() {
        let surface = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut input = PolyData::new();
        input.points.push([-1.0, 0.5, 0.0]); // off the left edge

        let result = project_to_surface(&input, &surface);
        let p = result.points.get(0);
        // Should project onto the edge from (0,0,0) to (0,1,0)
        assert!(p[0].abs() < 1e-10);
    }
}
