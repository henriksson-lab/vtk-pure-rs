use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the total surface area of a PolyData mesh.
pub fn compute_area(input: &PolyData) -> f64 {
    let mut total = 0.0;
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            total += triangle_area(v0, v1, v2);
        }
    }
    total
}

/// Compute per-cell areas and add as cell data "Area".
pub fn cell_areas(input: &PolyData) -> PolyData {
    let mut areas = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            areas.push(0.0);
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let mut cell_area = 0.0;
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            cell_area += triangle_area(v0, v1, v2);
        }
        areas.push(cell_area);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Area", areas, 1)));
    pd
}

/// Compute total line length.
pub fn compute_length(input: &PolyData) -> f64 {
    let mut total = 0.0;
    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() - 1 {
            let a = input.points.get(cell[i] as usize);
            let b = input.points.get(cell[i + 1] as usize);
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            total += (dx * dx + dy * dy + dz * dz).sqrt();
        }
    }
    total
}

fn triangle_area(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> f64 {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cx = e1[1] * e2[2] - e1[2] * e2[1];
    let cy = e1[2] * e2[0] - e1[0] * e2[2];
    let cz = e1[0] * e2[1] - e1[1] * e2[0];
    0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_triangle_area() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert!((compute_area(&pd) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn cell_areas_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = cell_areas(&pd);
        let arr = result.cell_data().get_array("Area").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn line_length() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([3.0, 4.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        assert!((compute_length(&pd) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        assert_eq!(compute_area(&pd), 0.0);
        assert_eq!(compute_length(&pd), 0.0);
    }
}
