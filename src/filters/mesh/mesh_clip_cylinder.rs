//! Clip mesh by infinite cylinder.
use crate::data::{CellArray, Points, PolyData};
pub fn clip_inside_cylinder(
    mesh: &PolyData,
    axis_origin: [f64; 3],
    axis_dir: [f64; 3],
    radius: f64,
) -> PolyData {
    clip_cyl(mesh, axis_origin, axis_dir, radius, true)
}
pub fn clip_outside_cylinder(
    mesh: &PolyData,
    axis_origin: [f64; 3],
    axis_dir: [f64; 3],
    radius: f64,
) -> PolyData {
    clip_cyl(mesh, axis_origin, axis_dir, radius, false)
}
fn clip_cyl(mesh: &PolyData, o: [f64; 3], d: [f64; 3], r: f64, inside: bool) -> PolyData {
    let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let dn = if dl < f64::EPSILON {
        [0.0, 1.0, 0.0]
    } else {
        [d[0] / dl, d[1] / dl, d[2] / dl]
    };
    let r2 = r * r;
    let mut used = vec![false; mesh.points.len()];
    let kept_verts = collect_kept_cells(&mesh.verts, mesh, o, dn, r2, inside, &mut used);
    let kept_lines = collect_kept_cells(&mesh.lines, mesh, o, dn, r2, inside, &mut used);
    let kept_polys = collect_kept_cells(&mesh.polys, mesh, o, dn, r2, inside, &mut used);
    let kept_strips = collect_kept_cells(&mesh.strips, mesh, o, dn, r2, inside, &mut used);

    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = remap_cells(&kept_verts, &pm);
    r.lines = remap_cells(&kept_lines, &pm);
    r.polys = remap_cells(&kept_polys, &pm);
    r.strips = remap_cells(&kept_strips, &pm);
    r
}

fn collect_kept_cells(
    cells: &CellArray,
    mesh: &PolyData,
    o: [f64; 3],
    dn: [f64; 3],
    r2: f64,
    inside: bool,
    used: &mut [bool],
) -> Vec<Vec<i64>> {
    let mut kept = Vec::new();
    for cell in cells.iter() {
        let Some([cx, cy, cz]) = cell_centroid(mesh, cell) else {
            continue;
        };
        let v = [cx - o[0], cy - o[1], cz - o[2]];
        let proj = v[0] * dn[0] + v[1] * dn[1] + v[2] * dn[2];
        let perp = [
            v[0] - proj * dn[0],
            v[1] - proj * dn[1],
            v[2] - proj * dn[2],
        ];
        let d2 = perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2];
        if (d2 <= r2) == inside {
            for &v in cell {
                used[v as usize] = true;
            }
            kept.push(cell.to_vec());
        }
    }
    kept
}

fn remap_cells(cells: &[Vec<i64>], pt_map: &[usize]) -> CellArray {
    let mut out = CellArray::new();
    for cell in cells {
        let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
        out.push_cell(&mapped);
    }
    out
}

fn cell_centroid(mesh: &PolyData, cell: &[i64]) -> Option<[f64; 3]> {
    if cell.is_empty()
        || cell
            .iter()
            .any(|&v| v < 0 || v as usize >= mesh.points.len())
    {
        return None;
    }
    let mut c = [0.0; 3];
    for &v in cell {
        let p = mesh.points.get(v as usize);
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = cell.len() as f64;
    Some([c[0] / n, c[1] / n, c[2] / n])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.5, 0.0, 0.0],
                [0.25, 0.5, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = clip_inside_cylinder(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
        assert_eq!(r.polys.num_cells(), 1);
    }

    #[test]
    fn zero_axis_uses_vtk_default_y_axis() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 10.0, 0.0],
                [0.1, 10.0, 0.0],
                [0.0, 10.0, 0.1],
                [10.0, 0.0, 0.0],
                [10.1, 0.0, 0.0],
                [10.0, 0.0, 0.1],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let r = clip_inside_cylinder(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);

        assert_eq!(r.polys.num_cells(), 1);
        assert_eq!(r.points.get(0), [0.0, 10.0, 0.0]);
    }
}
