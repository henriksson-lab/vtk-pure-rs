//! Butterfly subdivision (interpolating).
use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

pub fn subdivide_butterfly(mesh: &PolyData) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut new_polys = CellArray::new();
    let mut em: HashMap<(usize, usize), usize> = HashMap::new();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut ef: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    let mut point_cells: Vec<Vec<usize>> = vec![Vec::new(); mesh.points.len()];
    for (ci, cell) in cells.iter().enumerate() {
        if cell.len() != 3 || !valid_point_cell(cell, mesh.points.len()) {
            continue;
        }
        for &point_id in cell {
            point_cells[point_id as usize].push(ci);
        }
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    for cell in &cells {
        if cell.len() != 3 || !valid_point_cell(cell, mesh.points.len()) {
            new_polys.push_cell(cell);
            continue;
        }
        let v = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        let m20 = get_butterfly_mid(&mut pts, &mut em, &ef, &point_cells, &cells, v[2], v[0]);
        let m01 = get_butterfly_mid(&mut pts, &mut em, &ef, &point_cells, &cells, v[0], v[1]);
        let m12 = get_butterfly_mid(&mut pts, &mut em, &ef, &point_cells, &cells, v[1], v[2]);
        new_polys.push_cell(&[v[0] as i64, m01 as i64, m20 as i64]);
        new_polys.push_cell(&[m01 as i64, v[1] as i64, m12 as i64]);
        new_polys.push_cell(&[m12 as i64, v[2] as i64, m20 as i64]);
        new_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
    }
    let mut new_pts = Points::<f64>::new();
    for p in &pts {
        new_pts.push(*p);
    }
    let mut r = PolyData::new();
    r.points = new_pts;
    r.polys = new_polys;
    r
}

fn valid_point_cell(cell: &[i64], num_points: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < num_points)
}

fn get_butterfly_mid(
    pts: &mut Vec<[f64; 3]>,
    cache: &mut HashMap<(usize, usize), usize>,
    ef: &HashMap<(usize, usize), Vec<usize>>,
    point_cells: &[Vec<usize>],
    cells: &[Vec<i64>],
    a: usize,
    b: usize,
) -> usize {
    let key = (a.min(b), a.max(b));
    *cache.entry(key).or_insert_with(|| {
        let mid = butterfly_point(pts, ef, point_cells, cells, a, b);
        let i = pts.len();
        pts.push(mid);
        i
    })
}

fn butterfly_point(
    pts: &[[f64; 3]],
    ef: &HashMap<(usize, usize), Vec<usize>>,
    point_cells: &[Vec<usize>],
    cells: &[Vec<i64>],
    p1: usize,
    p2: usize,
) -> [f64; 3] {
    let faces = ef.get(&(p1.min(p2), p1.max(p2)));
    match faces.map(Vec::as_slice) {
        Some([_]) => {
            let (stencil, weights) = generate_boundary_stencil(ef, point_cells, cells, p1, p2);
            interpolate(pts, &stencil, &weights)
        }
        Some([cell0, cell1]) => {
            let valence1 = point_cells[p1].len();
            let valence2 = point_cells[p2].len();
            let (stencil, weights) = if valence1 == 6 && valence2 == 6 {
                generate_butterfly_stencil(ef, cells, *cell0, *cell1, p1, p2)
            } else if valence1 == 6 {
                generate_loop_stencil(ef, cells, p2, p1)
            } else if valence2 == 6 {
                generate_loop_stencil(ef, cells, p1, p2)
            } else {
                let (stencil1, weights1) = generate_loop_stencil(ef, cells, p2, p1);
                let (stencil2, weights2) = generate_loop_stencil(ef, cells, p1, p2);
                let mut stencil = Vec::with_capacity(stencil1.len() + stencil2.len());
                let mut weights = Vec::with_capacity(weights1.len() + weights2.len());
                for (&point_id, &weight) in stencil1.iter().zip(&weights1) {
                    stencil.push(point_id);
                    weights.push(weight * 0.5);
                }
                for (&point_id, &weight) in stencil2.iter().zip(&weights2) {
                    stencil.push(point_id);
                    weights.push(weight * 0.5);
                }
                (stencil, weights)
            };
            interpolate(pts, &stencil, &weights)
        }
        _ => midpoint(pts[p1], pts[p2]),
    }
}

fn generate_loop_stencil(
    ef: &HashMap<(usize, usize), Vec<usize>>,
    cells: &[Vec<i64>],
    p1: usize,
    p2: usize,
) -> (Vec<usize>, Vec<f64>) {
    let edge_faces = match ef.get(&(p1.min(p2), p1.max(p2))) {
        Some(faces) if faces.len() == 2 => faces,
        _ => return (vec![p1, p2], vec![0.5, 0.5]),
    };
    let start_cell = edge_faces[0];
    let mut next_cell = edge_faces[1];
    let mut tp2 = p2;
    let mut stencil = vec![p2];
    let mut shifts = vec![0isize];
    let mut processed = 0isize;

    while next_cell != start_cell {
        let Some(p) = opposite_vertex(&cells[next_cell], p1, tp2) else {
            return (vec![p1, p2], vec![0.5, 0.5]);
        };
        tp2 = p;
        stencil.push(tp2);
        processed += 1;
        shifts.push(processed);

        let next_edge_faces = match ef.get(&(p1.min(tp2), p1.max(tp2))) {
            Some(faces) => faces,
            None => return (vec![p1, p2], vec![0.5, 0.5]),
        };
        let adjacent: Vec<usize> = next_edge_faces
            .iter()
            .copied()
            .filter(|&cell_id| cell_id != next_cell)
            .collect();
        if adjacent.len() != 1 {
            let cell0 = start_cell;
            let cell1 = edge_faces[1];
            return generate_butterfly_stencil(ef, cells, cell0, cell1, p1, p2);
        }
        next_cell = adjacent[0];
        if stencil.len() > cells.len() + 1 {
            return (vec![p1, p2], vec![0.5, 0.5]);
        }
    }

    let k = stencil.len();
    let mut weights = vec![0.0; k + 1];
    if k >= 5 {
        for (j, shift) in shifts.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * (*shift as f64) / (k as f64);
            weights[j] = (0.25 + angle.cos() + 0.5 * (2.0 * angle).cos()) / (k as f64);
        }
    } else if k == 4 {
        let weights4 = [3.0 / 8.0, 0.0, -1.0 / 8.0, 0.0];
        for (j, shift) in shifts.iter().enumerate() {
            weights[j] = weights4[shift.unsigned_abs()];
        }
    } else if k == 3 {
        let weights3 = [5.0 / 12.0, -1.0 / 12.0, -1.0 / 12.0];
        for (j, shift) in shifts.iter().enumerate() {
            weights[j] = weights3[shift.unsigned_abs()];
        }
    } else {
        let p = opposite_vertex(&cells[start_cell], p1, p2).unwrap_or(p2);
        stencil.push(p);
        weights = vec![5.0 / 12.0, -1.0 / 12.0, -1.0 / 12.0, 0.75];
        stencil.push(p1);
        return (stencil, weights);
    }

    weights[k] = 0.75;
    stencil.push(p1);
    (stencil, weights)
}

fn generate_boundary_stencil(
    ef: &HashMap<(usize, usize), Vec<usize>>,
    point_cells: &[Vec<usize>],
    cells: &[Vec<i64>],
    p1: usize,
    p2: usize,
) -> (Vec<usize>, Vec<f64>) {
    let p0 = find_boundary_neighbor(ef, point_cells, cells, p1, &[p2]);
    if let Some(p0) = p0 {
        let p3 = find_boundary_neighbor(ef, point_cells, cells, p2, &[p1, p0]);
        if let Some(p3) = p3 {
            return (vec![p0, p1, p2, p3], vec![-0.0625, 0.5625, 0.5625, -0.0625]);
        }
        return (vec![p0, p1, p2], vec![-0.0625, 0.5625, 0.5625]);
    }
    (vec![p1, p2], vec![0.5, 0.5])
}

fn generate_butterfly_stencil(
    ef: &HashMap<(usize, usize), Vec<usize>>,
    cells: &[Vec<i64>],
    cell0: usize,
    cell1: usize,
    p1: usize,
    p2: usize,
) -> (Vec<usize>, Vec<f64>) {
    let p3 = opposite_vertex(&cells[cell0], p1, p2).unwrap_or(p1);
    let p4 = opposite_vertex(&cells[cell1], p1, p2).unwrap_or(p2);
    let p5 = opposite_across_edge(ef, cells, cell0, p1, p3).unwrap_or(p4);
    let p6 = opposite_across_edge(ef, cells, cell0, p2, p3).unwrap_or(p4);
    let p7 = opposite_across_edge(ef, cells, cell1, p1, p4).unwrap_or(p3);
    let p8 = opposite_across_edge(ef, cells, cell1, p2, p4).unwrap_or(p3);
    (
        vec![p1, p2, p3, p4, p5, p6, p7, p8],
        vec![0.5, 0.5, 0.125, 0.125, -0.0625, -0.0625, -0.0625, -0.0625],
    )
}

fn find_boundary_neighbor(
    ef: &HashMap<(usize, usize), Vec<usize>>,
    point_cells: &[Vec<usize>],
    cells: &[Vec<i64>],
    point: usize,
    excluded: &[usize],
) -> Option<usize> {
    for &cell_id in &point_cells[point] {
        for &candidate in &cells[cell_id] {
            let candidate = candidate as usize;
            if candidate == point || excluded.contains(&candidate) {
                continue;
            }
            if ef
                .get(&(point.min(candidate), point.max(candidate)))
                .map_or(0, Vec::len)
                == 1
            {
                return Some(candidate);
            }
        }
    }
    None
}

fn opposite_across_edge(
    ef: &HashMap<(usize, usize), Vec<usize>>,
    cells: &[Vec<i64>],
    from_cell: usize,
    a: usize,
    b: usize,
) -> Option<usize> {
    ef.get(&(a.min(b), a.max(b))).and_then(|faces| {
        faces
            .iter()
            .copied()
            .find(|&cell_id| cell_id != from_cell)
            .and_then(|cell_id| opposite_vertex(&cells[cell_id], a, b))
    })
}

fn opposite_vertex(cell: &[i64], a: usize, b: usize) -> Option<usize> {
    cell.iter()
        .map(|&id| id as usize)
        .find(|&id| id != a && id != b)
}

fn interpolate(pts: &[[f64; 3]], stencil: &[usize], weights: &[f64]) -> [f64; 3] {
    let mut p = [0.0; 3];
    for (&point_id, &weight) in stencil.iter().zip(weights) {
        let q = pts[point_id];
        p[0] += q[0] * weight;
        p[1] += q[1] * weight;
        p[2] += q[2] * weight;
    }
    p
}

fn midpoint(pa: [f64; 3], pb: [f64; 3]) -> [f64; 3] {
    [
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = subdivide_butterfly(&m);
        assert_eq!(r.polys.num_cells(), 4);
        assert_eq!(r.points.get(3), [0.4375, 1.125, 0.0]);
    }
}
