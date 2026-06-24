//! Compute mesh boundary perimeter length.
use crate::data::PolyData;
pub fn boundary_perimeter(mesh: &PolyData) -> f64 {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }
    ec.iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&(a, b), _)| {
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt()
        })
        .sum()
}
pub fn total_edge_length(mesh: &PolyData) -> f64 {
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut total = 0.0;
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if seen.insert((a.min(b), a.max(b))) {
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                total +=
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
            }
        }
    }
    total
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_perimeter() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let p = boundary_perimeter(&m);
        assert!(p > 2.0);
    }
    #[test]
    fn test_total() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let t = total_edge_length(&m);
        assert!((t - (1.0 + 1.0 + 2.0f64.sqrt())).abs() < 1e-10);
    }
}
