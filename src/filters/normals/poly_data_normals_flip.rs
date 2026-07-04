use crate::data::{CellArray, PolyData};

/// Auto-orient polygon normals after making winding consistent.
///
/// This mirrors the `vtkOrientPolyData`/`vtkPolyDataNormals` auto-orient
/// sequence more closely than independent face tests: first propagate
/// consistent winding across polygon neighbors, then flip each connected
/// closed-surface component if its signed volume is inward.
pub fn auto_orient_normals(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 || input.polys.num_cells() == 0 {
        return input.clone();
    }

    let mut pd = crate::filters::normals::orient::orient(input);
    let components = polygon_components(&pd.polys);
    let mut reverse = vec![false; pd.polys.num_cells()];
    for component in components {
        if signed_volume(&pd, &component) < 0.0 {
            for cell_id in component {
                reverse[cell_id] = true;
            }
        }
    }
    pd.polys = reverse_cells(&pd.polys, &reverse);
    pd
}

fn polygon_components(cells: &CellArray) -> Vec<Vec<usize>> {
    let nc = cells.num_cells();
    let offsets = cells.offsets();
    let conn = cells.connectivity();
    let mut edge_to_cells = std::collections::HashMap::<(i64, i64), Vec<usize>>::new();

    for ci in 0..nc {
        let start = offsets[ci] as usize;
        let end = offsets[ci + 1] as usize;
        let n = end - start;
        if n < 3 {
            continue;
        }
        for i in 0..n {
            let a = conn[start + i];
            let b = conn[start + if i + 1 < n { i + 1 } else { 0 }];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_cells.entry(key).or_default().push(ci);
        }
    }

    let mut adjacency = vec![Vec::new(); nc];
    for incident in edge_to_cells.values() {
        for (i, &a) in incident.iter().enumerate() {
            for &b in &incident[i + 1..] {
                adjacency[a].push(b);
                adjacency[b].push(a);
            }
        }
    }

    let mut visited = vec![false; nc];
    let mut components = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    for start in 0..nc {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        queue.push_back(start);
        let mut component = Vec::new();
        while let Some(cell_id) = queue.pop_front() {
            component.push(cell_id);
            for &neighbor in &adjacency[cell_id] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        components.push(component);
    }
    components
}

fn reverse_cells(cells: &CellArray, reverse: &[bool]) -> CellArray {
    let mut out = CellArray::new();
    for (cell_id, cell) in cells.iter().enumerate() {
        if reverse.get(cell_id).copied().unwrap_or(false) {
            let mut reversed = cell.to_vec();
            reversed.reverse();
            out.push_cell(&reversed);
        } else {
            out.push_cell(cell);
        }
    }
    out
}

fn signed_volume(poly_data: &PolyData, component: &[usize]) -> f64 {
    let mut volume = 0.0;
    for &cell_id in component {
        let cell = poly_data.polys.cell(cell_id);
        if cell.len() < 3
            || !cell
                .iter()
                .all(|&id| id >= 0 && (id as usize) < poly_data.points.len())
        {
            continue;
        }
        let p0 = poly_data.points.get(cell[0] as usize);
        for i in 1..(cell.len() - 1) {
            let p1 = poly_data.points.get(cell[i] as usize);
            let p2 = poly_data.points.get(cell[i + 1] as usize);
            volume += dot(p0, cross(p1, p2)) / 6.0;
        }
    }
    volume
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_outward() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // CCW = +Z normal, centroid below

        let result = auto_orient_normals(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn flips_inward() {
        // Create a box-like mesh where one face is flipped
        let mut pd = PolyData::new();
        // Simple case: two triangles, one with wrong winding
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]); // base
        pd.polys.push_cell(&[0, 1, 3]); // side

        let result = auto_orient_normals(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = auto_orient_normals(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn flips_disconnected_inward_component_independently() {
        let mut pd = PolyData::new();
        pd.points = crate::data::Points::from(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 0.0, 1.0],
        ]);
        for cell in [
            [0, 2, 1],
            [0, 1, 3],
            [1, 2, 3],
            [2, 0, 3],
            [4, 5, 6],
            [4, 7, 5],
            [5, 7, 6],
            [6, 7, 4],
        ] {
            pd.polys.push_cell(&cell);
        }

        let result = auto_orient_normals(&pd);
        let components = polygon_components(&result.polys);
        assert_eq!(components.len(), 2);
        assert!(components
            .iter()
            .all(|component| signed_volume(&result, component) > 0.0));
    }
}
