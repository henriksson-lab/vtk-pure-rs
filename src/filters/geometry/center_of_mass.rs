use crate::data::PolyData;

/// Compute the center of mass of a point set.
///
/// Returns the arithmetic mean of all point coordinates.
/// For a surface mesh weighted by area, use `mass_properties` instead.
pub fn center_of_mass(input: &PolyData) -> [f64; 3] {
    let n = input.points.len();
    if n == 0 {
        return [0.0, 0.0, 0.0];
    }

    let pts = input.points.as_flat_slice();
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let b = i * 3;
        cx += pts[b];
        cy += pts[b + 1];
        cz += pts[b + 2];
    }
    let nf = n as f64;
    [cx / nf, cy / nf, cz / nf]
}

/// Compute the area-weighted center of mass of a triangulated surface.
pub fn center_of_mass_area_weighted(input: &PolyData) -> [f64; 3] {
    let mut total_area = 0.0;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let p0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let area =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            let tc = [
                (p0[0] + p1[0] + p2[0]) / 3.0,
                (p0[1] + p1[1] + p2[1]) / 3.0,
                (p0[2] + p1[2] + p2[2]) / 3.0,
            ];
            cx += area * tc[0];
            cy += area * tc[1];
            cz += area * tc[2];
            total_area += area;
        }
    }

    if total_area > 1e-30 {
        [cx / total_area, cy / total_area, cz / total_area]
    } else {
        center_of_mass(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_of_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([0.0, 4.0, 0.0]);

        let c = center_of_mass(&pd);
        assert!((c[0] - 2.0 / 3.0).abs() < 1e-10);
        assert!((c[1] - 4.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn center_symmetric_mesh() {
        let pd = PolyData::from_triangles(
            vec![
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let c = center_of_mass_area_weighted(&pd);
        assert!(c[0].abs() < 1e-10);
        assert!(c[1].abs() < 1e-10);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let c = center_of_mass(&pd);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }
}
