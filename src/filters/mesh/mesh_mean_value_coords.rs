//! Mean value coordinates for mesh parameterization.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn mean_value_parameterize(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
                *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }
    // Find boundary and place on circle
    let mut boundary = Vec::new();
    let mut bset = std::collections::HashSet::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            bset.insert(a);
            bset.insert(b);
        }
    }
    if bset.is_empty() {
        return mesh.clone();
    }
    let mut badj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for (&(a, b), &c) in &ec {
        if c == 1 {
            badj.entry(a).or_default().push(b);
            badj.entry(b).or_default().push(a);
        }
    }
    let start = *bset.iter().next().unwrap();
    let mut cur = start;
    let mut visited = std::collections::HashSet::new();
    loop {
        boundary.push(cur);
        visited.insert(cur);
        let next = badj
            .get(&cur)
            .and_then(|nbs| nbs.iter().find(|&&n| !visited.contains(&n)));
        match next {
            Some(&n) => cur = n,
            None => {
                break;
            }
        }
    }
    let nb_len = boundary.len();
    if nb_len < 3 {
        return mesh.clone();
    }
    let mut uv = vec![[0.0f64; 2]; n];
    for (i, &vi) in boundary.iter().enumerate() {
        let a = 2.0 * std::f64::consts::PI * i as f64 / nb_len as f64;
        uv[vi] = [a.cos(), a.sin()];
    }
    let is_boundary: std::collections::HashSet<usize> = boundary.iter().copied().collect();
    // Mean value weight iteration
    for _ in 0..iterations {
        let prev = uv.clone();
        for i in 0..n {
            if is_boundary.contains(&i) || nb[i].is_empty() {
                continue;
            }
            // Mean value weights based on angles
            let p = mesh.points.get(i);
            let mut wsum = [0.0, 0.0];
            let mut wtotal = 0.0;
            for &j in &nb[i] {
                let pj = mesh.points.get(j);
                let d = ((p[0] - pj[0]).powi(2) + (p[1] - pj[1]).powi(2) + (p[2] - pj[2]).powi(2))
                    .sqrt()
                    .max(1e-15);
                let w = 1.0 / d; // simplified mean value weight
                wsum[0] += w * prev[j][0];
                wsum[1] += w * prev[j][1];
                wtotal += w;
            }
            if wtotal > 1e-15 {
                uv[i] = [wsum[0] / wtotal, wsum[1] / wtotal];
            }
        }
    }
    let data: Vec<f64> = uv.iter().flat_map(|p| vec![p[0], p[1]]).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("MVC_UV", data, 2)));
    r.point_data_mut().set_active_tcoords("MVC_UV");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = mean_value_parameterize(&m, 50);
        assert!(r.point_data().get_array("MVC_UV").is_some());
        assert_eq!(
            r.point_data().get_array("MVC_UV").unwrap().num_components(),
            2
        );
    }
}
