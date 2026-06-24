use std::collections::HashSet;

use crate::data::PolyData;
use rayon::prelude::*;

/// Laplacian smoothing of a PolyData mesh.
pub fn smooth(
    input: &PolyData,
    iterations: usize,
    relaxation_factor: f64,
    fix_boundary: bool,
) -> PolyData {
    let mut output = input.clone();
    let n = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    // Build adjacency as CSR (compressed sparse row) — flat arrays, cache-friendly
    let mut adj_count = vec![0u32; n];
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            adj_count[cell[j] as usize] += (len - 1) as u32; // upper bound
        }
    }
    let mut adj_offset = vec![0u32; n + 1];
    for i in 0..n {
        adj_offset[i + 1] = adj_offset[i] + adj_count[i];
    }
    let total_adj = adj_offset[n] as usize;
    let mut adj_list = vec![0u32; total_adj];
    let mut adj_pos = adj_offset[..n].to_vec(); // write cursor per vertex

    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            for k in 0..len {
                if k == j {
                    continue;
                }
                let b = cell[k] as u32;
                let pos = adj_pos[a] as usize;
                adj_list[pos] = b;
                adj_pos[a] += 1;
            }
        }
    }
    // Deduplicate each vertex's neighbor list in-place
    for i in 0..n {
        let start = adj_offset[i] as usize;
        let end = adj_pos[i] as usize;
        let slice = &mut adj_list[start..end];
        slice.sort_unstable();
        let mut write = 0;
        for read in 0..slice.len() {
            if read == 0 || slice[read] != slice[read - 1] {
                slice[write] = slice[read];
                write += 1;
            }
        }
        adj_count[i] = write as u32;
    }
    // Rebuild offsets after dedup
    adj_offset[0] = 0;
    for i in 0..n {
        adj_offset[i + 1] = adj_offset[i] + adj_count[i];
    }

    // Boundary detection using edge counting with flat array
    let boundary: Vec<bool> = if fix_boundary {
        find_boundary_fast(input, n)
    } else {
        vec![false; n]
    };

    let factor = relaxation_factor.clamp(0.0, 1.0);

    // Work directly on flat f64 buffer for cache efficiency
    let mut pos: Vec<f64> = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = output.points.get(i);
        pos.extend_from_slice(&p);
    }
    let mut new_pos = vec![0.0f64; n * 3];

    for _ in 0..iterations {
        for i in 0..n {
            let px = pos[i * 3];
            let py = pos[i * 3 + 1];
            let pz = pos[i * 3 + 2];

            if boundary[i] || adj_count[i] == 0 {
                new_pos[i * 3] = px;
                new_pos[i * 3 + 1] = py;
                new_pos[i * 3 + 2] = pz;
                continue;
            }

            let start = adj_offset[i] as usize;
            let count = adj_count[i] as usize;
            let mut ax = 0.0f64;
            let mut ay = 0.0f64;
            let mut az = 0.0f64;
            for j in 0..count {
                let nb = unsafe { *adj_list.get_unchecked(start + j) } as usize;
                unsafe {
                    ax += *pos.get_unchecked(nb * 3);
                    ay += *pos.get_unchecked(nb * 3 + 1);
                    az += *pos.get_unchecked(nb * 3 + 2);
                }
            }
            let inv = 1.0 / count as f64;
            ax *= inv;
            ay *= inv;
            az *= inv;

            new_pos[i * 3] = px + factor * (ax - px);
            new_pos[i * 3 + 1] = py + factor * (ay - py);
            new_pos[i * 3 + 2] = pz + factor * (az - pz);
        }
        std::mem::swap(&mut pos, &mut new_pos);
    }

    // Write back
    for i in 0..n {
        output
            .points
            .set(i, [pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]]);
    }
    output
}

/// Parallel Laplacian smoothing using rayon.
pub fn smooth_par(
    input: &PolyData,
    iterations: usize,
    relaxation_factor: f64,
    fix_boundary: bool,
) -> PolyData {
    let mut output = input.clone();
    let n = output.points.len();
    if n == 0 || iterations == 0 {
        return output;
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            let b = cell[(j + 1) % len] as usize;
            neighbors[a].push(b);
            neighbors[b].push(a);
        }
    }
    for nbrs in &mut neighbors {
        nbrs.sort_unstable();
        nbrs.dedup();
    }

    let boundary = if fix_boundary {
        find_boundary_vertices(input)
    } else {
        HashSet::new()
    };
    let factor = relaxation_factor.clamp(0.0, 1.0);

    for _ in 0..iterations {
        let current: Vec<[f64; 3]> = (0..n).map(|i| output.points.get(i)).collect();
        let new_positions: Vec<[f64; 3]> = (0..n)
            .into_par_iter()
            .map(|i| {
                let p = current[i];
                let nbrs = &neighbors[i];
                if boundary.contains(&i) || nbrs.is_empty() {
                    return p;
                }
                let mut avg = [0.0f64; 3];
                let count = nbrs.len() as f64;
                for &nb in nbrs {
                    let q = current[nb];
                    avg[0] += q[0];
                    avg[1] += q[1];
                    avg[2] += q[2];
                }
                avg[0] /= count;
                avg[1] /= count;
                avg[2] /= count;
                [
                    p[0] + factor * (avg[0] - p[0]),
                    p[1] + factor * (avg[1] - p[1]),
                    p[2] + factor * (avg[2] - p[2]),
                ]
            })
            .collect();
        for (i, &pos) in new_positions.iter().enumerate() {
            output.points.set(i, pos);
        }
    }
    output
}

/// Fast boundary detection using flat edge count array.
fn find_boundary_fast(input: &PolyData, n: usize) -> Vec<bool> {
    // Count edge occurrences using a hash-free approach:
    // For each directed edge (a,b) where a<b, count occurrences.
    // Boundary edges have count == 1.
    use std::collections::HashMap;
    let mut edge_count: HashMap<u64, u8> = HashMap::new();
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            let b = cell[(j + 1) % len] as usize;
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            let key = (lo as u64) << 32 | hi as u64;
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let mut boundary = vec![false; n];
    for (&key, &count) in &edge_count {
        if count == 1 {
            boundary[(key >> 32) as usize] = true;
            boundary[(key & 0xFFFFFFFF) as usize] = true;
        }
    }
    boundary
}

fn find_boundary_vertices(input: &PolyData) -> HashSet<usize> {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in input.polys.iter() {
        let len = cell.len();
        for j in 0..len {
            let a = cell[j] as usize;
            let b = cell[(j + 1) % len] as usize;
            let edge = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    }
    let mut boundary = HashSet::new();
    for (&(a, b), &count) in &edge_count {
        if count == 1 {
            boundary.insert(a);
            boundary.insert(b);
        }
    }
    boundary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_preserves_topology() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = smooth(&pd, 5, 0.5, false);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn smooth_moves_interior() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.5, 0.5, 0.0],
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        );
        let before = pd.points.get(4);
        let result = smooth(&pd, 10, 0.5, false);
        let after = result.points.get(4);
        assert!(
            (after[0] - 1.0).abs() < (before[0] - 1.0).abs(),
            "x should move toward center"
        );
    }

    #[test]
    fn zero_iterations_noop() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = smooth(&pd, 0, 0.5, false);
        assert_eq!(result.points.get(0), pd.points.get(0));
    }
}
