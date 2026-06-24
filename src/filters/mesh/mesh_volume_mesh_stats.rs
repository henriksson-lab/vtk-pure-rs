//! Comprehensive volumetric mesh statistics.
use crate::data::PolyData;
pub struct VolumeMeshStats {
    pub num_vertices: usize,
    pub num_edges: usize,
    pub num_faces: usize,
    pub surface_area: f64,
    pub volume: f64,
    pub compactness: f64,
    pub aspect_ratio: f64,
    pub bounds: ([f64; 3], [f64; 3]),
    pub centroid: [f64; 3],
    pub is_closed: bool,
    pub euler: isize,
}
pub fn compute_volume_mesh_stats(mesh: &PolyData) -> VolumeMeshStats {
    let nv = mesh.points.len();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let mut area = 0.0;
    let mut vol = 0.0;
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..nv {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    if nv > 0 {
        let nf = nv as f64;
        cx /= nf;
        cy /= nf;
        cz /= nf;
    }
    let nf = mesh.polys.num_cells();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
        if nc >= 3 {
            let a = mesh.points.get(cell[0] as usize);
            for i in 1..nc - 1 {
                let b = mesh.points.get(cell[i] as usize);
                let c = mesh.points.get(cell[i + 1] as usize);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let cx2 = e1[1] * e2[2] - e1[2] * e2[1];
                let cy2 = e1[2] * e2[0] - e1[0] * e2[2];
                let cz2 = e1[0] * e2[1] - e1[1] * e2[0];
                area += 0.5 * (cx2 * cx2 + cy2 * cy2 + cz2 * cz2).sqrt();
                vol += a[0] * (b[1] * c[2] - b[2] * c[1])
                    + a[1] * (b[2] * c[0] - b[0] * c[2])
                    + a[2] * (b[0] * c[1] - b[1] * c[0]);
            }
        }
    }
    vol = vol.abs() / 6.0;
    let ne = ec.len();
    let boundary = ec.values().filter(|&&c| c == 1).count();
    let euler = nv as isize - ne as isize + nf as isize;
    let diag = ((mx[0] - mn[0]).powi(2) + (mx[1] - mn[1]).powi(2) + (mx[2] - mn[2]).powi(2)).sqrt();
    let dims = [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]];
    let max_dim = dims[0].max(dims[1]).max(dims[2]).max(1e-15);
    let min_dim = dims[0].min(dims[1]).min(dims[2]).max(1e-15);
    let compact = if area > 1e-15 {
        36.0 * std::f64::consts::PI * vol * vol / (area * area * area)
    } else {
        0.0
    };
    if nv == 0 {
        mn = [0.0; 3];
        mx = [0.0; 3];
    }
    VolumeMeshStats {
        num_vertices: nv,
        num_edges: ne,
        num_faces: nf,
        surface_area: area,
        volume: vol,
        compactness: compact,
        aspect_ratio: max_dim / min_dim,
        bounds: (mn, mx),
        centroid: [cx, cy, cz],
        is_closed: boundary == 0,
        euler,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let s = compute_volume_mesh_stats(&m);
        assert_eq!(s.num_vertices, 4);
        assert_eq!(s.num_faces, 4);
        assert!(s.volume > 0.0);
        assert!(s.is_closed);
        assert_eq!(s.euler, 2);
    }
}
