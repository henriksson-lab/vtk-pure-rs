use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the star (link) of each vertex: ordered ring of neighbor vertices.
///
/// For each vertex, returns the degree (number of edges) and whether
/// the vertex is on the boundary. Adds "StarDegree" and "IsBoundary" arrays.
pub fn vertex_star_info(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); n];
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            neighbors[a].insert(b);
            neighbors[b].insert(a);
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    let mut degrees = vec![0.0f64; n];
    let mut is_boundary = vec![0.0f64; n];

    for i in 0..n {
        degrees[i] = neighbors[i].len() as f64;
    }

    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            is_boundary[a] = 1.0;
            is_boundary[b] = 1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "StarDegree",
            degrees,
            1,
        )));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IsBoundary",
            is_boundary,
            1,
        )));
    pd
}

/// Count the number of boundary vertices.
pub fn count_boundary_vertices(input: &PolyData) -> usize {
    let result = vertex_star_info(input);
    let arr = match result.point_data().get_array("IsBoundary") {
        Some(a) => a,
        None => return 0,
    };
    let mut buf = [0.0f64];
    (0..arr.num_tuples())
        .filter(|&i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .count()
}

/// Count the number of interior vertices.
pub fn count_interior_vertices(input: &PolyData) -> usize {
    input.points.len() - count_boundary_vertices(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_degrees() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }

        let result = vertex_star_info(&pd);
        let arr = result.point_data().get_array("StarDegree").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 6.0); // center has 6 neighbors
    }

    #[test]
    fn boundary_detection() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert_eq!(count_boundary_vertices(&pd), 3); // all boundary (single triangle)
        assert_eq!(count_interior_vertices(&pd), 0);
    }

    #[test]
    fn closed_mesh_no_boundary() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        assert_eq!(count_boundary_vertices(&pd), 0);
        assert_eq!(count_interior_vertices(&pd), 4);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(count_boundary_vertices(&pd), 0);
    }
}
