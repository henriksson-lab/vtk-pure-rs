use crate::data::{AnyDataArray, DataArray, PolyData};

/// Approximate geodesic distance using the heat method.
///
/// 1. Diffuse a heat pulse from source vertices
/// 2. Compute normalized gradient of the heat
/// 3. Solve a Poisson equation to recover distance
///
/// Simplified version: uses the heat diffusion step directly
/// (accurate for short distances). Adds "HeatDistance" scalar.
pub fn heat_method_distance(input: &PolyData, sources: &[usize], diffusion_time: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
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
        }
    }

    // Step 1: Diffuse heat
    let mut heat = vec![0.0f64; n];
    for &s in sources {
        if s < n {
            heat[s] = 1.0;
        }
    }

    let dt = diffusion_time.max(0.01);
    let steps = (dt / 0.1).ceil() as usize;
    for _ in 0..steps {
        let mut new = heat.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let avg: f64 =
                neighbors[i].iter().map(|&j| heat[j]).sum::<f64>() / neighbors[i].len() as f64;
            new[i] = heat[i] + 0.1 * (avg - heat[i]);
        }
        heat = new;
    }

    // Step 2: Convert heat to distance (approximate: -log(heat) * scale)
    let max_heat = heat.iter().copied().fold(0.0f64, f64::max);
    let dist: Vec<f64> = heat
        .iter()
        .map(|&h| {
            if h > 1e-15 && max_heat > 1e-15 {
                -(h / max_heat).ln() * dt.sqrt()
            } else {
                f64::MAX
            }
        })
        .collect();

    // Normalize so source = 0
    let min_dist = dist.iter().copied().fold(f64::MAX, f64::min);
    let result: Vec<f64> = dist
        .iter()
        .map(|&d| if d < f64::MAX { d - min_dist } else { -1.0 })
        .collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HeatDistance",
            result,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_zero_distance() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = heat_method_distance(&pd, &[0], 1.0);
        let arr = result.point_data().get_array("HeatDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0]).abs() < 1e-5); // source = 0
    }

    #[test]
    fn distance_increases() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = heat_method_distance(&pd, &[0], 1.0);
        let arr = result.point_data().get_array("HeatDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let d0 = buf[0];
        arr.tuple_as_f64(2, &mut buf);
        let d2 = buf[0];
        assert!(d2 > d0); // farther = larger distance
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = heat_method_distance(&pd, &[0], 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
