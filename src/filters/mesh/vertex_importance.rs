use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute vertex importance for mesh simplification.
///
/// Combines curvature magnitude, boundary distance, and valence deviation
/// into a single importance score. Higher = more important to preserve.
/// Adds "Importance" scalar.
pub fn vertex_importance(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count = std::collections::HashMap::new();
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
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0usize) += 1;
        }
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    // Factor 1: Curvature (Laplacian magnitude)
    let mut curvature = vec![0.0f64; n];
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let p = pts[i];
        let cnt = neighbors[i].len() as f64;
        let mut lx = 0.0;
        let mut ly = 0.0;
        let mut lz = 0.0;
        for &j in &neighbors[i] {
            lx += pts[j][0] - p[0];
            ly += pts[j][1] - p[1];
            lz += pts[j][2] - p[2];
        }
        curvature[i] = (lx * lx + ly * ly + lz * lz).sqrt() / cnt;
    }

    // Factor 2: Is boundary vertex?
    let mut is_boundary = vec![0.0f64; n];
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            is_boundary[a] = 1.0;
            is_boundary[b] = 1.0;
        }
    }

    // Factor 3: Valence deviation from 6
    let mut val_dev = vec![0.0f64; n];
    for i in 0..n {
        val_dev[i] = (neighbors[i].len() as f64 - 6.0).abs() / 6.0;
    }

    // Normalize each factor to [0,1]
    let normalize = |v: &mut [f64]| {
        let mx = v.iter().copied().fold(0.0f64, f64::max);
        if mx > 1e-15 {
            for x in v.iter_mut() {
                *x /= mx;
            }
        }
    };
    normalize(&mut curvature);
    normalize(&mut val_dev);

    // Combined importance = 0.5*curvature + 0.3*boundary + 0.2*valence_dev
    let importance: Vec<f64> = (0..n)
        .map(|i| 0.5 * curvature[i] + 0.3 * is_boundary[i] + 0.2 * val_dev[i])
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Importance",
            importance,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_higher_importance() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }

        let result = vertex_importance(&pd);
        let arr = result.point_data().get_array("Importance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let center = buf[0];
        arr.tuple_as_f64(1, &mut buf);
        let boundary = buf[0];
        assert!(boundary > center); // boundary vertices more important
    }

    #[test]
    fn has_importance_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = vertex_importance(&pd);
        assert!(result.point_data().get_array("Importance").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = vertex_importance(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
