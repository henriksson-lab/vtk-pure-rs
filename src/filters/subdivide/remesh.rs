use crate::data::{CellArray, Points, PolyData};
use std::collections::{HashMap, HashSet};

/// Simple isotropic remeshing via iterative edge split/collapse/flip.
///
/// Aims for a target edge length. Each pass:
/// 1. Split edges longer than 4/3 * target
/// 2. Collapse edges shorter than 4/5 * target
/// 3. Flip edges to improve valence toward 6
/// 4. Tangential smoothing
///
pub fn remesh(input: &PolyData, target_edge_length: f64, iterations: usize) -> PolyData {
    if target_edge_length <= 0.0 {
        return input.clone();
    }

    let mut points: Vec<[f64; 3]> = (0..input.points.len())
        .map(|i| input.points.get(i))
        .collect();
    let mut tris: Vec<[i64; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();

    if tris.is_empty() {
        return input.clone();
    }

    let high = target_edge_length * 4.0 / 3.0;
    let low = target_edge_length * 4.0 / 5.0;

    for _ in 0..iterations {
        split_long_edges(&mut points, &mut tris, high);
        collapse_short_edges(&mut points, &mut tris, low);
        flip_edges_for_valence(&mut tris);
        tangential_relax(&mut points, &tris);
    }

    // Build output
    let mut used: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for tri in &tris {
        let mapped: Vec<i64> = tri
            .iter()
            .map(|&id| {
                *used.entry(id).or_insert_with(|| {
                    let idx = out_points.len() as i64;
                    out_points.push(points[id as usize]);
                    idx
                })
            })
            .collect();
        out_polys.push_cell(&mapped);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn split_long_edges(points: &mut Vec<[f64; 3]>, tris: &mut Vec<[i64; 3]>, high: f64) {
    let mut new_tris = Vec::new();
    let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();

    for &[a, b, c] in tris.iter() {
        let edges = [(a, b), (b, c), (c, a)];
        let lengths = [
            edge_length(points, a, b),
            edge_length(points, b, c),
            edge_length(points, c, a),
        ];
        let longest = if lengths[0] >= lengths[1] && lengths[0] >= lengths[2] {
            0
        } else if lengths[1] >= lengths[2] {
            1
        } else {
            2
        };

        if lengths[longest] <= high {
            new_tris.push([a, b, c]);
            continue;
        }

        let (u, v) = edges[longest];
        let m = get_mid(points, &mut midpoint_cache, u, v);
        let w = [a, b, c]
            .iter()
            .copied()
            .find(|&id| id != u && id != v)
            .unwrap();
        new_tris.push([u, m, w]);
        new_tris.push([m, v, w]);
    }

    *tris = new_tris;
}

fn get_mid(
    points: &mut Vec<[f64; 3]>,
    cache: &mut HashMap<(i64, i64), i64>,
    a: i64,
    b: i64,
) -> i64 {
    let key = if a < b { (a, b) } else { (b, a) };
    *cache.entry(key).or_insert_with(|| {
        let pa = points[a as usize];
        let pb = points[b as usize];
        let idx = points.len() as i64;
        points.push([
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ]);
        idx
    })
}

fn collapse_short_edges(points: &mut [[f64; 3]], tris: &mut Vec<[i64; 3]>, low: f64) {
    let mut remap: Vec<usize> = (0..points.len()).collect();
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
    let mut edge_list = Vec::new();

    for tri in tris.iter() {
        for i in 0..3 {
            let a = tri[i] as usize;
            let b = tri[(i + 1) % 3] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            let count = edge_counts.entry(key).or_insert(0);
            if *count == 0 {
                edge_list.push(key);
            }
            *count += 1;
        }
    }

    edge_list.sort_by(|&(a0, b0), &(a1, b1)| {
        edge_length_usize(points, a0, b0)
            .partial_cmp(&edge_length_usize(points, a1, b1))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut collapsed = HashSet::new();
    for (a, b) in edge_list {
        if edge_counts.get(&(a, b)).copied().unwrap_or(0) != 2 {
            continue;
        }

        let ra = find_root(&remap, a);
        let rb = find_root(&remap, b);
        if ra == rb || collapsed.contains(&ra) || collapsed.contains(&rb) {
            continue;
        }

        if edge_length_usize(points, ra, rb) < low {
            points[ra] = [
                (points[ra][0] + points[rb][0]) * 0.5,
                (points[ra][1] + points[rb][1]) * 0.5,
                (points[ra][2] + points[rb][2]) * 0.5,
            ];
            remap[rb] = ra;
            collapsed.insert(ra);
            collapsed.insert(rb);
        }
    }

    tris.retain_mut(|tri| {
        let a = find_root(&remap, tri[0] as usize) as i64;
        let b = find_root(&remap, tri[1] as usize) as i64;
        let c = find_root(&remap, tri[2] as usize) as i64;
        *tri = [a, b, c];
        a != b && b != c && a != c
    });
}

fn flip_edges_for_valence(tris: &mut [[i64; 3]]) {
    let mut edge_tris: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (ti, tri) in tris.iter().enumerate() {
        for i in 0..3 {
            let a = tri[i];
            let b = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_tris.entry(key).or_default().push(ti);
        }
    }

    let mut valence: HashMap<i64, usize> = HashMap::new();
    for tri in tris.iter() {
        for &v in tri {
            *valence.entry(v).or_insert(0) += 1;
        }
    }

    let target_valence = 6_i64;
    for (&(a, b), face_list) in &edge_tris {
        if face_list.len() != 2 {
            continue;
        }

        let ti0 = face_list[0];
        let ti1 = face_list[1];
        let Some(c) = tris[ti0].iter().copied().find(|&v| v != a && v != b) else {
            continue;
        };
        let Some(d) = tris[ti1].iter().copied().find(|&v| v != a && v != b) else {
            continue;
        };
        if c == d {
            continue;
        }

        let va = *valence.get(&a).unwrap_or(&0) as i64;
        let vb = *valence.get(&b).unwrap_or(&0) as i64;
        let vc = *valence.get(&c).unwrap_or(&0) as i64;
        let vd = *valence.get(&d).unwrap_or(&0) as i64;
        let before = (va - target_valence).abs()
            + (vb - target_valence).abs()
            + (vc - target_valence).abs()
            + (vd - target_valence).abs();
        let after = (va - 1 - target_valence).abs()
            + (vb - 1 - target_valence).abs()
            + (vc + 1 - target_valence).abs()
            + (vd + 1 - target_valence).abs();

        if after < before {
            tris[ti0] = [a, d, c];
            tris[ti1] = [b, c, d];
            *valence.entry(a).or_insert(0) -= 1;
            *valence.entry(b).or_insert(0) -= 1;
            *valence.entry(c).or_insert(0) += 1;
            *valence.entry(d).or_insert(0) += 1;
        }
    }
}

fn tangential_relax(points: &mut Vec<[f64; 3]>, tris: &[[i64; 3]]) {
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); points.len()];
    for tri in tris {
        for i in 0..3 {
            let a = tri[i] as usize;
            neighbors[a].insert(tri[(i + 1) % 3] as usize);
            neighbors[a].insert(tri[(i + 2) % 3] as usize);
        }
    }

    let old_points = points.clone();
    for i in 0..points.len() {
        if neighbors[i].is_empty() {
            continue;
        }

        let count = neighbors[i].len() as f64;
        let mut average = [0.0, 0.0, 0.0];
        for &neighbor in &neighbors[i] {
            average[0] += old_points[neighbor][0];
            average[1] += old_points[neighbor][1];
            average[2] += old_points[neighbor][2];
        }
        average[0] /= count;
        average[1] /= count;
        average[2] /= count;

        points[i][0] += (average[0] - old_points[i][0]) * 0.5;
        points[i][1] += (average[1] - old_points[i][1]) * 0.5;
        points[i][2] += (average[2] - old_points[i][2]) * 0.5;
    }
}

fn find_root(remap: &[usize], mut v: usize) -> usize {
    while remap[v] != v {
        v = remap[v];
    }
    v
}

fn edge_length(points: &[[f64; 3]], a: i64, b: i64) -> f64 {
    edge_length_usize(points, a as usize, b as usize)
}

fn edge_length_usize(points: &[[f64; 3]], a: usize, b: usize) -> f64 {
    let pa = points[a];
    let pb = points[b];
    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refines_large_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remesh(&pd, 3.0, 1);
        assert!(result.polys.num_cells() > 1);
        assert!(result.points.len() > 3);
    }

    #[test]
    fn small_triangle_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.05, 0.1, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remesh(&pd, 1.0, 5);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = remesh(&pd, 1.0, 5);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
