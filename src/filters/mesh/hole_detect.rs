use crate::data::{AnyDataArray, DataArray, PolyData};
use std::collections::HashMap;

/// Detect holes in a mesh and compute their perimeter lengths.
///
/// Adds "HoleId" point data: 0 for non-boundary vertices, positive
/// integer for each boundary loop. Returns (PolyData, Vec<f64>) where
/// the Vec contains the perimeter of each hole.
pub fn detect_holes(input: &PolyData) -> (PolyData, Vec<f64>) {
    let n = input.points.len();
    if n == 0 {
        return (input.clone(), vec![]);
    }

    let mut edge_count: HashMap<(i64, i64), usize> = HashMap::new();
    let mut boundary_adj: HashMap<i64, Vec<i64>> = HashMap::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary_adj.entry(a).or_default().push(b);
            boundary_adj.entry(b).or_default().push(a);
        }
    }

    let mut hole_ids = vec![0.0f64; n];
    let mut visited = std::collections::HashSet::new();
    let mut perimeters = Vec::new();
    let mut current_hole = 0;

    for &start in boundary_adj.keys() {
        if visited.contains(&start) {
            continue;
        }
        current_hole += 1;
        let mut perimeter = 0.0;
        let mut cur = start;

        loop {
            if visited.contains(&cur) {
                break;
            }
            visited.insert(cur);
            hole_ids[cur as usize] = current_hole as f64;

            let nexts = boundary_adj.get(&cur);
            let next = nexts.and_then(|v| v.iter().find(|&&n| !visited.contains(&n)));
            match next {
                Some(&nc) => {
                    let pa = input.points.get(cur as usize);
                    let pb = input.points.get(nc as usize);
                    perimeter += ((pa[0] - pb[0]).powi(2)
                        + (pa[1] - pb[1]).powi(2)
                        + (pa[2] - pb[2]).powi(2))
                    .sqrt();
                    cur = nc;
                }
                None => break,
            }
        }
        perimeters.push(perimeter);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HoleId", hole_ids, 1,
        )));
    (pd, perimeters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_hole() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (result, perimeters) = detect_holes(&pd);
        assert!(result.point_data().get_array("HoleId").is_some());
        assert_eq!(perimeters.len(), 1);
        assert!(perimeters[0] > 0.0);
    }

    #[test]
    fn closed_mesh_no_holes() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let (_, perimeters) = detect_holes(&pd);
        assert_eq!(perimeters.len(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (_, perimeters) = detect_holes(&pd);
        assert!(perimeters.is_empty());
    }
}
