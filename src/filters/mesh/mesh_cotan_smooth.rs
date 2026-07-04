//! Cotangent-weighted Laplacian smoothing.
use crate::data::PolyData;

pub fn cotan_smooth(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter_map(|c| triangle_point_ids(c, n))
        .collect();
    let mut positions: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            [p[0], p[1], p[2]]
        })
        .collect();
    for _ in 0..iterations {
        let mut lap = vec![[0.0f64; 3]; n];
        let mut weight_sum = vec![0.0f64; n];
        for &[a, b, c] in &tris {
            if a >= n || b >= n || c >= n {
                continue;
            }
            add_cot_weight(&positions, a, b, c, &mut lap, &mut weight_sum);
            add_cot_weight(&positions, b, c, a, &mut lap, &mut weight_sum);
            add_cot_weight(&positions, c, a, b, &mut lap, &mut weight_sum);
        }
        let mut new_pos = positions.clone();
        for i in 0..n {
            if weight_sum[i] > 1e-15 {
                for d in 0..3 {
                    new_pos[i][d] += lambda * lap[i][d] / weight_sum[i];
                }
            }
        }
        positions = new_pos;
    }
    let mut result = mesh.clone();
    for (i, p) in positions.iter().enumerate() {
        result.points.set(i, *p);
    }
    result
}

fn triangle_point_ids(cell: &[i64], n: usize) -> Option<[usize; 3]> {
    if cell.len() != 3 {
        return None;
    }
    Some([
        valid_point_id(cell[0], n)?,
        valid_point_id(cell[1], n)?,
        valid_point_id(cell[2], n)?,
    ])
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    if id >= 0 && (id as usize) < n {
        Some(id as usize)
    } else {
        None
    }
}

fn add_cot_weight(
    positions: &[[f64; 3]],
    i: usize,
    j: usize,
    opposite: usize,
    lap: &mut [[f64; 3]],
    weight_sum: &mut [f64],
) {
    let u = [
        positions[i][0] - positions[opposite][0],
        positions[i][1] - positions[opposite][1],
        positions[i][2] - positions[opposite][2],
    ];
    let v = [
        positions[j][0] - positions[opposite][0],
        positions[j][1] - positions[opposite][1],
        positions[j][2] - positions[opposite][2],
    ];
    let dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let cross_len = ((u[1] * v[2] - u[2] * v[1]).powi(2)
        + (u[2] * v[0] - u[0] * v[2]).powi(2)
        + (u[0] * v[1] - u[1] * v[0]).powi(2))
    .sqrt();
    let w = if cross_len > 1e-15 {
        (dot / cross_len).clamp(-100.0, 100.0)
    } else {
        0.0
    };
    for d in 0..3 {
        lap[i][d] += w * (positions[j][d] - positions[i][d]);
        lap[j][d] += w * (positions[i][d] - positions[j][d]);
    }
    let abs_w = w.abs();
    weight_sum[i] += abs_w;
    weight_sum[j] += abs_w;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cotan_smooth() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.3],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = cotan_smooth(&mesh, 3, 0.5);
        assert_eq!(r.points.len(), 4);
    }
}
