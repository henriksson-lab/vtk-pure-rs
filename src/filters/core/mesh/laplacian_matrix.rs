use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the Laplacian coordinates (delta coordinates) at each vertex.
///
/// Delta = vertex_position - average_of_neighbors. These encode local
/// shape detail and are used in Laplacian mesh editing. Adds "DeltaX",
/// "DeltaY", "DeltaZ" scalar arrays.
pub fn laplacian_coordinates(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 { return input.clone(); }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize; let b = cell[(i+1)%cell.len()] as usize;
            if a < n && b < n {
                if !neighbors[a].contains(&b) { neighbors[a].push(b); }
                if !neighbors[b].contains(&a) { neighbors[b].push(a); }
            }
        }
    }

    let mut dx = vec![0.0f64; n];
    let mut dy = vec![0.0f64; n];
    let mut dz = vec![0.0f64; n];

    for i in 0..n {
        let p = input.points.get(i);
        if neighbors[i].is_empty() { continue; }
        let cnt = neighbors[i].len() as f64;
        let mut ax = 0.0; let mut ay = 0.0; let mut az = 0.0;
        for &j in &neighbors[i] {
            let q = input.points.get(j);
            ax += q[0]; ay += q[1]; az += q[2];
        }
        dx[i] = p[0] - ax/cnt;
        dy[i] = p[1] - ay/cnt;
        dz[i] = p[2] - az/cnt;
    }

    let mut pd = input.clone();
    pd.point_data_mut().add_array(AnyDataArray::F64(DataArray::from_vec("DeltaX", dx, 1)));
    pd.point_data_mut().add_array(AnyDataArray::F64(DataArray::from_vec("DeltaY", dy, 1)));
    pd.point_data_mut().add_array(AnyDataArray::F64(DataArray::from_vec("DeltaZ", dz, 1)));
    pd
}

/// Compute the Laplacian magnitude (norm of delta coordinates).
/// High values indicate high local curvature/detail.
pub fn laplacian_magnitude(input: &PolyData) -> PolyData {
    let result = laplacian_coordinates(input);
    let dx = match result.point_data().get_array("DeltaX") { Some(a) => a, None => return input.clone() };
    let dy = result.point_data().get_array("DeltaY").unwrap();
    let dz = result.point_data().get_array("DeltaZ").unwrap();

    let n = dx.num_tuples();
    let mut buf = [0.0f64];
    let mut mag = Vec::with_capacity(n);
    for i in 0..n {
        dx.tuple_as_f64(i, &mut buf); let x = buf[0];
        dy.tuple_as_f64(i, &mut buf); let y = buf[0];
        dz.tuple_as_f64(i, &mut buf); let z = buf[0];
        mag.push((x*x+y*y+z*z).sqrt());
    }

    let mut pd = result;
    pd.point_data_mut().add_array(AnyDataArray::F64(DataArray::from_vec("LaplacianMag", mag, 1)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_zero_laplacian() {
        let mut pd = PolyData::new();
        for j in 0..3 { for i in 0..3 { pd.points.push([i as f64, j as f64, 0.0]); }}
        for j in 0..2 { for i in 0..2 {
            let a = (j*3+i) as i64;
            pd.polys.push_cell(&[a,a+1,a+4]);
            pd.polys.push_cell(&[a,a+4,a+3]);
        }}

        let result = laplacian_coordinates(&pd);
        let arr = result.point_data().get_array("DeltaZ").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf); // center
        assert!(buf[0].abs() < 1e-10);
    }

    #[test]
    fn magnitude_positive() {
        let mut pd = PolyData::new();
        pd.points.push([0.0,0.0,0.0]); pd.points.push([1.0,0.0,0.0]);
        pd.points.push([0.5,1.0,0.5]); // off-plane
        pd.polys.push_cell(&[0,1,2]);

        let result = laplacian_magnitude(&pd);
        assert!(result.point_data().get_array("LaplacianMag").is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = laplacian_coordinates(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
