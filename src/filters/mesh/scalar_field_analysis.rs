//! Scalar field analysis on meshes: critical points, gradient lines, level sets.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::{HashMap, HashSet, VecDeque};

/// Find critical points (minima, maxima, saddles) of a scalar field on a mesh.
pub fn find_scalar_critical_points(mesh: &PolyData, array_name: &str) -> PolyData {
    let n = mesh.points.len();
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return PolyData::new(),
    };
    let (adj, links) = build_link_topology(mesh, n);
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut crit_pts = Points::<f64>::new();
    let mut crit_type = Vec::new(); // 0=min, 1=max, 2=saddle

    for i in 0..n {
        if adj[i].is_empty() {
            continue;
        }
        let vi = values[i];
        let lower: Vec<usize> = adj[i].iter().copied().filter(|&j| values[j] < vi).collect();
        let higher: Vec<usize> = adj[i].iter().copied().filter(|&j| values[j] > vi).collect();

        if lower.is_empty() && !higher.is_empty() {
            // local minimum
            crit_pts.push(mesh.points.get(i));
            crit_type.push(0.0);
        } else if higher.is_empty() && !lower.is_empty() {
            // local maximum
            crit_pts.push(mesh.points.get(i));
            crit_type.push(1.0);
        } else if link_components(&links[i], &higher) > 1 || link_components(&links[i], &lower) > 1
        {
            crit_pts.push(mesh.points.get(i));
            crit_type.push(2.0);
        }
    }

    let mut result = PolyData::new();
    result.points = crit_pts;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CriticalType",
            crit_type,
            1,
        )));
    result
}

/// Extract level set (iso-contour) of a scalar field on a mesh.
pub fn extract_level_set(mesh: &PolyData, array_name: &str, isovalue: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= mesh.points.len() => a,
        _ => return PolyData::new(),
    };
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();

    for cell in mesh.polys.iter() {
        if cell.len() < 3
            || cell
                .iter()
                .any(|&id| id < 0 || id as usize >= mesh.points.len())
        {
            continue;
        }
        let mut crossings = Vec::new();
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let va = values[a];
            let vb = values[b];
            if (va - isovalue) * (vb - isovalue) < 0.0 {
                let t = (isovalue - va) / (vb - va);
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                crossings.push([
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    pa[2] + t * (pb[2] - pa[2]),
                ]);
            }
        }
        if crossings.len() >= 2 {
            let i0 = pts.len() as i64;
            pts.push(crossings[0]);
            let i1 = pts.len() as i64;
            pts.push(crossings[1]);
            lines.push_cell(&[i0, i1]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Compute scalar gradient on mesh as per-vertex vectors.
///
/// Thin wrapper over
/// [`crate::filters::mesh::point_data_gradient::scalar_gradient_on_mesh`], which owns the
/// single per-face-gradient implementation. The result is republished under this
/// module's "Gradient" array name.
pub fn scalar_gradient_on_mesh(mesh: &PolyData, array_name: &str) -> PolyData {
    let mut result =
        crate::filters::mesh::point_data_gradient::scalar_gradient_on_mesh(mesh, array_name);
    let Some(gradient) = result.point_data_mut().remove_array("ScalarGradient") else {
        return result;
    };
    let _ = result.point_data_mut().remove_array("GradientMagnitude");
    let values = gradient.to_f64_vec_flat();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Gradient", values, 3,
        )));
    result
}

fn build_link_topology(
    mesh: &PolyData,
    n: usize,
) -> (Vec<Vec<usize>>, Vec<HashMap<usize, Vec<usize>>>) {
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut links: Vec<HashMap<usize, Vec<usize>>> = vec![HashMap::new(); n];

    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            if cell[i] < 0 || cell[(i + 1) % cell.len()] < 0 {
                continue;
            }
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }

        for i in 0..cell.len() {
            if cell[i] < 0
                || cell[(i + cell.len() - 1) % cell.len()] < 0
                || cell[(i + 1) % cell.len()] < 0
            {
                continue;
            }
            let center = cell[i] as usize;
            let prev = cell[(i + cell.len() - 1) % cell.len()] as usize;
            let next = cell[(i + 1) % cell.len()] as usize;
            if center >= n || prev >= n || next >= n || prev == next {
                continue;
            }
            let prev_link = links[center].entry(prev).or_default();
            if !prev_link.contains(&next) {
                prev_link.push(next);
            }
            let next_link = links[center].entry(next).or_default();
            if !next_link.contains(&prev) {
                next_link.push(prev);
            }
        }
    }

    (neighbors, links)
}

fn link_components(link: &HashMap<usize, Vec<usize>>, subset: &[usize]) -> usize {
    if subset.is_empty() {
        return 0;
    }

    let subset_set: HashSet<usize> = subset.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut components = 0usize;

    for &start in subset {
        if !seen.insert(start) {
            continue;
        }

        components += 1;
        let mut queue = VecDeque::from([start]);
        while let Some(v) = queue.pop_front() {
            for &next in link.get(&v).into_iter().flatten() {
                if subset_set.contains(&next) && seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }

    components
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn critical_points() {
        let mut pts = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        // Scalar: distance from center → one minimum at center
        let vals: Vec<f64> = (0..100)
            .map(|i| {
                let x = (i % 10) as f64 - 4.5;
                let y = (i / 10) as f64 - 4.5;
                x * x + y * y
            })
            .collect();
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vals, 1)));
        let crits = find_scalar_critical_points(&mesh, "f");
        assert!(crits.points.len() > 0);
    }
    #[test]
    fn level_set() {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mut mesh = PolyData::from_triangles(pts, tris);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                (0..25).map(|i| i as f64).collect(),
                1,
            )));
        let ls = extract_level_set(&mesh, "f", 12.5);
        assert!(ls.lines.num_cells() > 0);
    }
    #[test]
    fn gradient() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0, 0.0, 1.0],
                1,
            )));
        let result = scalar_gradient_on_mesh(&mesh, "f");
        assert!(result.point_data().get_array("Gradient").is_some());
    }

    #[test]
    fn gradient_short_array_returns_input() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 1.0],
                1,
            )));

        let result = scalar_gradient_on_mesh(&mesh, "f");
        assert!(result.point_data().get_array("Gradient").is_none());
    }
}
