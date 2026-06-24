use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Extract a single connected component by index.
///
/// Uses union-find to label connected components, then extracts the
/// component with the given `component_id` (0-indexed, ordered by size
/// descending — so 0 is the largest component).
pub fn extract_component(input: &PolyData, component_id: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    // Union-find
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    let find = |parent: &mut Vec<usize>, mut x: usize| -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    };

    let union = |parent: &mut Vec<usize>, rank: &mut Vec<usize>, a: usize, b: usize| {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra == rb {
            return;
        }
        if rank[ra] < rank[rb] {
            parent[ra] = rb;
        } else if rank[ra] > rank[rb] {
            parent[rb] = ra;
        } else {
            parent[rb] = ra;
            rank[ra] += 1;
        }
    };

    // Connect vertices in each cell
    for cell in input.polys.iter() {
        for i in 1..cell.len() {
            union(&mut parent, &mut rank, cell[0] as usize, cell[i] as usize);
        }
    }
    for cell in input.lines.iter() {
        for i in 1..cell.len() {
            union(&mut parent, &mut rank, cell[0] as usize, cell[i] as usize);
        }
    }

    // Count component sizes
    let mut comp_sizes: HashMap<usize, usize> = HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        *comp_sizes.entry(root).or_insert(0) += 1;
    }

    // Sort by size descending
    let mut components: Vec<(usize, usize)> = comp_sizes.into_iter().collect();
    components.sort_by(|a, b| b.1.cmp(&a.1));

    if component_id >= components.len() {
        return PolyData::new();
    }

    let target_root = components[component_id].0;

    // Collect points belonging to target component
    let mut point_map: HashMap<usize, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();

    for i in 0..n {
        if find(&mut parent, i) == target_root {
            let idx = out_points.len() as i64;
            out_points.push(input.points.get(i));
            point_map.insert(i, idx);
        }
    }

    // Remap cells
    let mut out_polys = CellArray::new();
    for cell in input.polys.iter() {
        if cell
            .iter()
            .all(|&id| point_map.contains_key(&(id as usize)))
        {
            let mapped: Vec<i64> = cell.iter().map(|&id| point_map[&(id as usize)]).collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut out_lines = CellArray::new();
    for cell in input.lines.iter() {
        if cell
            .iter()
            .all(|&id| point_map.contains_key(&(id as usize)))
        {
            let mapped: Vec<i64> = cell.iter().map(|&id| point_map[&(id as usize)]).collect();
            out_lines.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd
}

/// Count the number of connected components.
pub fn num_components(input: &PolyData) -> usize {
    let n = input.points.len();
    if n == 0 {
        return 0;
    }

    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank: Vec<usize> = vec![0; n];

    let find = |parent: &mut Vec<usize>, mut x: usize| -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    };

    let union = |parent: &mut Vec<usize>, rank: &mut Vec<usize>, a: usize, b: usize| {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra == rb {
            return;
        }
        if rank[ra] < rank[rb] {
            parent[ra] = rb;
        } else if rank[ra] > rank[rb] {
            parent[rb] = ra;
        } else {
            parent[rb] = ra;
            rank[ra] += 1;
        }
    };

    for cell in input.polys.iter() {
        for i in 1..cell.len() {
            union(&mut parent, &mut rank, cell[0] as usize, cell[i] as usize);
        }
    }
    for cell in input.lines.iter() {
        for i in 1..cell.len() {
            union(&mut parent, &mut rank, cell[0] as usize, cell[i] as usize);
        }
    }

    let mut roots = std::collections::HashSet::new();
    for i in 0..n {
        roots.insert(find(&mut parent, i));
    }
    roots.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_components() {
        let mut pd = PolyData::new();
        // Component 0: triangle
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        // Component 1: triangle
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([5.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        assert_eq!(num_components(&pd), 2);

        let c0 = extract_component(&pd, 0);
        assert_eq!(c0.points.len(), 3);
        assert_eq!(c0.polys.num_cells(), 1);

        let c1 = extract_component(&pd, 1);
        assert_eq!(c1.points.len(), 3);
        assert_eq!(c1.polys.num_cells(), 1);
    }

    #[test]
    fn single_component() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert_eq!(num_components(&pd), 1);
        let c0 = extract_component(&pd, 0);
        assert_eq!(c0.polys.num_cells(), 1);
    }

    #[test]
    fn invalid_component_id() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = extract_component(&pd, 5);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(num_components(&pd), 0);
    }
}
