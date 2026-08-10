use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the Laplacian coordinates (delta coordinates) at each vertex.
///
/// Delta = vertex_position - average_of_neighbors. These encode local
/// shape detail and are used in Laplacian mesh editing. Adds "DeltaX",
/// "DeltaY", "DeltaZ" scalar arrays.
pub fn laplacian_coordinates(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            add_neighbor_pair(&mut neighbors, n, cell[i], cell[(i + 1) % cell.len()]);
        }
    }
    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_pair(&mut neighbors, n, edge[0], edge[1]);
        }
    }
    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            add_neighbor_pair(&mut neighbors, n, tri[0], tri[1]);
            add_neighbor_pair(&mut neighbors, n, tri[1], tri[2]);
            add_neighbor_pair(&mut neighbors, n, tri[2], tri[0]);
        }
    }

    let mut dx = vec![0.0f64; n];
    let mut dy = vec![0.0f64; n];
    let mut dz = vec![0.0f64; n];

    for i in 0..n {
        let p = input.points.get(i);
        if neighbors[i].is_empty() {
            continue;
        }
        let cnt = neighbors[i].len() as f64;
        let mut ax = 0.0;
        let mut ay = 0.0;
        let mut az = 0.0;
        for &j in &neighbors[i] {
            let q = input.points.get(j);
            ax += q[0];
            ay += q[1];
            az += q[2];
        }
        dx[i] = p[0] - ax / cnt;
        dy[i] = p[1] - ay / cnt;
        dz[i] = p[2] - az / cnt;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("DeltaX", dx, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("DeltaY", dy, 1)));
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("DeltaZ", dz, 1)));
    pd
}

pub use crate::filters::mesh::mesh_laplacian_vector::laplacian_magnitude;

fn add_neighbor_pair(neighbors: &mut [Vec<usize>], n: usize, a_id: i64, b_id: i64) {
    if a_id < 0 || b_id < 0 {
        return;
    }
    let a = a_id as usize;
    let b = b_id as usize;
    if a >= n || b >= n || a == b {
        return;
    }
    if !neighbors[a].contains(&b) {
        neighbors[a].push(b);
    }
    if !neighbors[b].contains(&a) {
        neighbors[b].push(a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_zero_laplacian() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let result = laplacian_coordinates(&pd);
        let arr = result.point_data().get_array("DeltaZ").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf); // center
        assert!(buf[0].abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = laplacian_coordinates(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
