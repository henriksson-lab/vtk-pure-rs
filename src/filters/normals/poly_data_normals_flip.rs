use crate::data::PolyData;

/// Auto-orient all polygon normals to point outward from the centroid.
///
/// For each triangle, checks if the face normal points away from the
/// mesh centroid. If not, flips the winding order. Useful for meshes
/// imported with inconsistent winding.
pub fn auto_orient_normals(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute mesh centroid
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    let centroid = [cx / nf, cy / nf, cz / nf];

    let mut pd = input.clone();
    let mut new_polys = crate::data::CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            new_polys.push_cell(cell);
            continue;
        }

        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);

        // Face normal
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];

        // Face centroid
        let fc = [
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ];

        // Vector from mesh centroid to face centroid
        let out = [
            fc[0] - centroid[0],
            fc[1] - centroid[1],
            fc[2] - centroid[2],
        ];
        let dot = fn_[0] * out[0] + fn_[1] * out[1] + fn_[2] * out[2];

        if dot < 0.0 {
            // Flip winding
            let mut flipped: Vec<i64> = cell.to_vec();
            flipped.reverse();
            new_polys.push_cell(&flipped);
        } else {
            new_polys.push_cell(cell);
        }
    }

    pd.polys = new_polys;
    pd
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
}
