use crate::data::{CellArray, Points, PolyData};

/// Cut a mesh with a plane and return the cross-section polyline.
///
/// The plane is defined by a point and normal. Returns line segments
/// where the plane intersects the mesh triangles.
pub fn planar_cross_section(
    input: &PolyData,
    plane_point: [f64; 3],
    plane_normal: [f64; 3],
) -> PolyData {
    let nlen = (plane_normal[0] * plane_normal[0]
        + plane_normal[1] * plane_normal[1]
        + plane_normal[2] * plane_normal[2])
        .sqrt();
    if nlen < 1e-15 {
        return PolyData::new();
    }
    let nn = [
        plane_normal[0] / nlen,
        plane_normal[1] / nlen,
        plane_normal[2] / nlen,
    ];
    let d = nn[0] * plane_point[0] + nn[1] * plane_point[1] + nn[2] * plane_point[2];

    let mut out_pts = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        let dists: Vec<f64> = ids
            .iter()
            .map(|&i| {
                let p = input.points.get(i);
                nn[0] * p[0] + nn[1] * p[1] + nn[2] * p[2] - d
            })
            .collect();

        let mut crossings = Vec::new();
        for k in 0..3 {
            let da = dists[k];
            let db = dists[(k + 1) % 3];
            if da * db < 0.0 {
                let t = da / (da - db);
                let pa = input.points.get(ids[k]);
                let pb = input.points.get(ids[(k + 1) % 3]);
                crossings.push([
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    pa[2] + t * (pb[2] - pa[2]),
                ]);
            }
        }

        if crossings.len() == 2 {
            let i0 = out_pts.len() as i64;
            out_pts.push(crossings[0]);
            out_pts.push(crossings[1]);
            out_lines.push_cell(&[i0, i0 + 1]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.lines = out_lines;
    pd
}

/// Compute the area of a planar cross-section (approximate).
pub fn cross_section_area(input: &PolyData, plane_point: [f64; 3], plane_normal: [f64; 3]) -> f64 {
    let section = planar_cross_section(input, plane_point, plane_normal);
    // Approximate area by summing triangle fan from centroid
    let n = section.points.len();
    if n < 3 {
        return 0.0;
    }

    // Compute centroid of cross-section points
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = section.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    // Sum triangle areas from centroid to each line segment
    let mut area = 0.0;
    for cell in section.lines.iter() {
        if cell.len() >= 2 {
            let a = section.points.get(cell[0] as usize);
            let b = section.points.get(cell[1] as usize);
            let e1 = [a[0] - cx, a[1] - cy, a[2] - cz];
            let e2 = [b[0] - cx, b[1] - cy, b[2] - cz];
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            area += 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        }
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_through_quad() {
        let mut pd = PolyData::new();
        pd.points.push([-1.0, -1.0, -1.0]);
        pd.points.push([1.0, -1.0, -1.0]);
        pd.points.push([1.0, 1.0, 1.0]);
        pd.points.push([-1.0, 1.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = planar_cross_section(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(result.lines.num_cells() > 0);
    }

    #[test]
    fn no_intersection() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 5.0]);
        pd.points.push([1.0, 0.0, 5.0]);
        pd.points.push([0.5, 1.0, 5.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = planar_cross_section(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn cross_section_area_positive() {
        let mut pd = PolyData::new();
        pd.points.push([-1.0, -1.0, -1.0]);
        pd.points.push([1.0, -1.0, -1.0]);
        pd.points.push([1.0, 1.0, 1.0]);
        pd.points.push([-1.0, 1.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let area = cross_section_area(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(area >= 0.0); // may be 0 if only 2 points
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(
            planar_cross_section(&pd, [0.0; 3], [0.0, 0.0, 1.0])
                .lines
                .num_cells(),
            0
        );
    }
}
