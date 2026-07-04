use crate::data::{DataArray, Points, PolyData};

/// Offset a mesh surface by moving each vertex along its normal by a given distance.
///
/// Computes per-vertex normals (averaged from incident face normals) and displaces
/// each point by `distance` along its normal. Returns a new surface with the same
/// connectivity but shifted positions. Unlike extrusion, no side walls are created.
pub fn offset_surface_simple(input: &PolyData, distance: f64) -> PolyData {
    let npts: usize = input.points.len();
    if npts == 0 {
        return input.clone();
    }

    // Compute face normals and accumulate at vertices.
    let mut normals: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; npts];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !cell.iter().all(|&id| valid_point_index(id, npts).is_some()) {
            continue;
        }
        let p0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let nx: f64 = e1[1] * e2[2] - e1[2] * e2[1];
            let ny: f64 = e1[2] * e2[0] - e1[0] * e2[2];
            let nz: f64 = e1[0] * e2[1] - e1[1] * e2[0];

            for &vid in &[cell[0], cell[i], cell[i + 1]] {
                let v: usize = vid as usize;
                normals[v][0] += nx;
                normals[v][1] += ny;
                normals[v][2] += nz;
            }
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
            if !tri.iter().all(|&id| valid_point_index(id, npts).is_some()) {
                continue;
            }
            let p0 = input.points.get(tri[0] as usize);
            let p1 = input.points.get(tri[1] as usize);
            let p2 = input.points.get(tri[2] as usize);
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let nx: f64 = e1[1] * e2[2] - e1[2] * e2[1];
            let ny: f64 = e1[2] * e2[0] - e1[0] * e2[2];
            let nz: f64 = e1[0] * e2[1] - e1[1] * e2[0];

            for &vid in &tri {
                let v: usize = vid as usize;
                normals[v][0] += nx;
                normals[v][1] += ny;
                normals[v][2] += nz;
            }
        }
    }

    // Normalize and offset.
    let mut out_points: Points<f64> = Points::new();
    let mut normal_arr = DataArray::<f64>::new("Normals", 3);
    for i in 0..npts {
        let n = normals[i];
        let len: f64 = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let (ux, uy, uz) = if len > 1e-15 {
            (n[0] / len, n[1] / len, n[2] / len)
        } else {
            (0.0, 0.0, 0.0)
        };
        let p = input.points.get(i);
        out_points.push([
            p[0] + ux * distance,
            p[1] + uy * distance,
            p[2] + uz * distance,
        ]);
        normal_arr.push_tuple(&[ux, uy, uz]);
    }

    let mut output = input.clone();
    output.points = out_points;
    output.point_data_mut().add_array(normal_arr.into());
    output.point_data_mut().set_active_normals("Normals");
    output
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CellArray;

    fn make_triangle() -> PolyData {
        let mut points: Points<f64> = Points::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.0, 1.0, 0.0]);
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);
        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        pd
    }

    #[test]
    fn offset_moves_along_normal() {
        let tri = make_triangle();
        let result = offset_surface_simple(&tri, 1.0);
        // Triangle in XY plane has normal along +Z.
        // All points should have z = 1.0 after offset.
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            assert!((p[2] - 1.0).abs() < 1e-10, "Point {} z = {}", i, p[2]);
        }
    }

    #[test]
    fn negative_offset() {
        let tri = make_triangle();
        let result = offset_surface_simple(&tri, -0.5);
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            assert!((p[2] - (-0.5)).abs() < 1e-10);
        }
    }

    #[test]
    fn connectivity_preserved() {
        let tri = make_triangle();
        let result = offset_surface_simple(&tri, 1.0);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn isolated_point_does_not_move() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);

        let result = offset_surface_simple(&pd, 1.0);

        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }
}
