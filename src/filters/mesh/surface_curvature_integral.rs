use crate::data::PolyData;

/// Compute total absolute curvature: integral of |H| over the surface.
///
/// Approximated by summing per-vertex curvature × Voronoi area.
pub fn total_absolute_curvature(input: &PolyData) -> f64 {
    let n = input.points.len();
    if n == 0 {
        return 0.0;
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut vertex_area = vec![0.0f64; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            let a = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt() / 3.0;
            vertex_area[cell[0] as usize] += a;
            vertex_area[cell[i] as usize] += a;
            vertex_area[cell[i + 1] as usize] += a;
        }
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut total = 0.0;
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let p = input.points.get(i);
        let cnt = neighbors[i].len() as f64;
        let mut lx = 0.0;
        let mut ly = 0.0;
        let mut lz = 0.0;
        for &j in &neighbors[i] {
            let q = input.points.get(j);
            lx += q[0] - p[0];
            ly += q[1] - p[1];
            lz += q[2] - p[2];
        }
        let curv = (lx * lx + ly * ly + lz * lz).sqrt() / cnt;
        total += curv * vertex_area[i];
    }
    total
}

/// Compute total surface area.
pub fn total_surface_area(input: &PolyData) -> f64 {
    let mut area = 0.0;
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }
    }
    area
}

/// Compute mean curvature: total_absolute_curvature / total_area.
pub fn mean_total_curvature(input: &PolyData) -> f64 {
    let a = total_surface_area(input);
    if a > 1e-15 {
        total_absolute_curvature(input) / a
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_low_curvature() {
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

        let mtc = mean_total_curvature(&pd);
        assert!(mtc < 1.0); // flat surface has low curvature
    }

    #[test]
    fn area_positive() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        assert!((total_surface_area(&pd) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(total_absolute_curvature(&pd), 0.0);
        assert_eq!(total_surface_area(&pd), 0.0);
    }
}
