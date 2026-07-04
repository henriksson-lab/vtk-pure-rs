//! Create offset surface along vertex normals.
use crate::data::{CellArray, Points, PolyData};
pub fn offset_surface(mesh: &PolyData, distance: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let nm: Vec<[f64; 3]> = if let Some(arr) = mesh
        .point_data()
        .get_array("Normals")
        .filter(|arr| arr.num_components() >= 3 && arr.num_tuples() == n)
    {
        let mut buf = [0.0f64; 3];
        (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf
            })
            .collect()
    } else {
        calc_normals(mesh)
    };
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + nm[i][0] * distance,
            p[1] + nm[i][1] * distance,
            p[2] + nm[i][2] * distance,
        ]);
    }
    let mut r = mesh.clone();
    r.points = pts;
    r
}
pub fn shell(mesh: &PolyData, inner_offset: f64, outer_offset: f64) -> PolyData {
    let n = mesh.points.len();
    let nm = calc_normals(mesh);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + nm[i][0] * inner_offset,
            p[1] + nm[i][1] * inner_offset,
            p[2] + nm[i][2] * inner_offset,
        ]);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + nm[i][0] * outer_offset,
            p[1] + nm[i][1] * outer_offset,
            p[2] + nm[i][2] * outer_offset,
        ]);
    }
    // Inner surface (reversed)
    for cell in mesh.polys.iter() {
        let mut rev: Vec<i64> = cell.to_vec();
        rev.reverse();
        polys.push_cell(&rev);
    }
    // Outer surface
    for cell in mesh.polys.iter() {
        let shifted: Vec<i64> = cell.iter().map(|&v| v + n as i64).collect();
        polys.push_cell(&shifted);
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            polys.push_cell(&[tri[2], tri[1], tri[0]]);
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            polys.push_cell(&[tri[0] + n as i64, tri[1] + n as i64, tri[2] + n as i64]);
        }
    }
    // Side walls on boundary edges
    let mut ec: std::collections::HashMap<(usize, usize), (usize, usize, usize)> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc == 0 {
            continue;
        }
        for i in 0..nc {
            count_directed_edge(cell[i], cell[(i + 1) % nc], n, &mut ec);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            count_directed_edge(tri[0], tri[1], n, &mut ec);
            count_directed_edge(tri[1], tri[2], n, &mut ec);
            count_directed_edge(tri[2], tri[0], n, &mut ec);
        }
    }
    for &(_, a, b) in ec.values() {
        let c = ec[&(a.min(b), a.max(b))].0;
        if c == 1 {
            polys.push_cell(&[a as i64, b as i64, (b + n) as i64, (a + n) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
fn calc_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        for i in 1..cell.len() - 1 {
            accumulate_triangle_normal(mesh, [cell[0], cell[i], cell[i + 1]], &mut nm);
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            accumulate_triangle_normal(mesh, tri, &mut nm);
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}
fn accumulate_triangle_normal(mesh: &PolyData, tri: [i64; 3], nm: &mut [[f64; 3]]) {
    let n = nm.len();
    let Some(ia) = valid_point_id(tri[0], n) else {
        return;
    };
    let Some(ib) = valid_point_id(tri[1], n) else {
        return;
    };
    let Some(ic) = valid_point_id(tri[2], n) else {
        return;
    };
    let a = mesh.points.get(ia);
    let b = mesh.points.get(ib);
    let c = mesh.points.get(ic);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let fn_ = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    for vi in [ia, ib, ic] {
        nm[vi][0] += fn_[0];
        nm[vi][1] += fn_[1];
        nm[vi][2] += fn_[2];
    }
}
fn count_directed_edge(
    a_id: i64,
    b_id: i64,
    n: usize,
    edge_count: &mut std::collections::HashMap<(usize, usize), (usize, usize, usize)>,
) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    if a == b {
        return;
    }
    let entry = edge_count.entry((a.min(b), a.max(b))).or_insert((0, a, b));
    entry.0 += 1;
}
fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_offset() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = offset_surface(&m, 0.1);
        assert_eq!(r.points.len(), 3);
    }
    #[test]
    fn test_shell() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = shell(&m, -0.05, 0.05);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() > 2);
    }
    #[test]
    fn offset_uses_strip_normals() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.strips.push_cell(&[0, 1, 2]);

        let r = offset_surface(&m, 0.1);
        assert!((r.points.get(0)[2] - 0.1).abs() < 1e-10);
    }
    #[test]
    fn shell_includes_strip_surface() {
        let mut m = PolyData::new();
        m.points.push([0.0, 0.0, 0.0]);
        m.points.push([1.0, 0.0, 0.0]);
        m.points.push([0.0, 1.0, 0.0]);
        m.strips.push_cell(&[0, 1, 2]);

        let r = shell(&m, -0.05, 0.05);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 5);
    }
}
