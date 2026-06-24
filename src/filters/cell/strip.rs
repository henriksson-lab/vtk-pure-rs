use crate::data::PolyData;
use std::collections::HashMap;

/// Convert triangle polygons to triangle strips.
///
/// Existing strips are passed through, non-triangle polygons stay in `polys`,
/// and triangle polygons are greedily assembled into new strips.
pub fn to_triangle_strips(input: &PolyData) -> PolyData {
    let mut output = PolyData::new();
    output.points = input.points.clone();
    output.verts = input.verts.clone();
    output.lines = input.lines.clone();
    output.strips = input.strips.clone();
    *output.point_data_mut() = input.point_data().clone();

    let mut tri_verts = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() == 3 {
            tri_verts.push([cell[0], cell[1], cell[2]]);
        } else {
            output.polys.push_cell(cell);
        }
    }

    let nt = tri_verts.len();
    if nt == 0 {
        return output;
    }

    let mut edge_tris: HashMap<(i64, i64), [usize; 2]> = HashMap::with_capacity(nt * 3);
    for (ti, tri) in tri_verts.iter().enumerate() {
        for edge in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = edge_key(edge.0, edge.1);
            let entry = edge_tris.entry(key).or_insert([usize::MAX, usize::MAX]);
            if entry[0] == usize::MAX {
                entry[0] = ti;
            } else if entry[1] == usize::MAX {
                entry[1] = ti;
            }
        }
    }

    let mut visited = vec![false; nt];
    for start in 0..nt {
        if visited[start] {
            continue;
        }
        visited[start] = true;

        let tri = tri_verts[start];
        let mut strip = vec![tri[0], tri[1], tri[2]];

        loop {
            let n = strip.len();
            let Some(next) = find_unvisited_neighbor(
                &tri_verts,
                &edge_tris,
                &visited,
                strip[n - 2],
                strip[n - 1],
            ) else {
                break;
            };

            visited[next.0] = true;
            strip.push(next.1);
        }

        output.strips.push_cell(&strip);
    }

    output
}

fn find_unvisited_neighbor(
    tri_verts: &[[i64; 3]],
    edge_tris: &HashMap<(i64, i64), [usize; 2]>,
    visited: &[bool],
    a: i64,
    b: i64,
) -> Option<(usize, i64)> {
    let pair = edge_tris.get(&edge_key(a, b))?;
    for &ti in pair {
        if ti == usize::MAX || visited[ti] {
            continue;
        }
        let tri = tri_verts[ti];
        for v in tri {
            if v != a && v != b {
                return Some((ti, v));
            }
        }
    }
    None
}

fn edge_key(a: i64, b: i64) -> (i64, i64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_to_strip() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = to_triangle_strips(&pd);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.strips.num_cells(), 1);
        assert_eq!(result.strips.cell(0).len(), 3);
    }

    #[test]
    fn two_adjacent_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = to_triangle_strips(&pd);
        let total_strip_verts: usize = result.strips.iter().map(|s| s.len()).sum();
        assert!(total_strip_verts <= 5);
    }

    #[test]
    fn preserves_points() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = to_triangle_strips(&pd);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn passes_existing_strips_and_non_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = to_triangle_strips(&pd);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.strips.num_cells(), 1);
    }
}
