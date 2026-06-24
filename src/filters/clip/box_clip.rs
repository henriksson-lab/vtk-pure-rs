use crate::data::{CellArray, Points, PolyData};

/// Clip PolyData by an axis-aligned bounding box.
///
/// `bounds` is `[xmin, xmax, ymin, ymax, zmin, zmax]`.
pub fn box_clip(input: &PolyData, bounds: [f64; 6]) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let mut polygon: Vec<[f64; 3]> = cell
            .iter()
            .map(|&pid| input.points.get(pid as usize))
            .collect();
        for plane in 0..6 {
            polygon = clip_polygon_to_box_plane(&polygon, bounds, plane);
            if polygon.len() < 3 {
                break;
            }
        }
        if polygon.len() < 3 {
            continue;
        }

        let base = out_points.len() as i64;
        for p in &polygon {
            out_points.push(*p);
        }
        for i in 1..polygon.len() - 1 {
            out_polys.push_cell(&[base, base + i as i64, base + (i + 1) as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.polys = out_polys;
    result
}

fn clip_polygon_to_box_plane(
    polygon: &[[f64; 3]],
    bounds: [f64; 6],
    plane: usize,
) -> Vec<[f64; 3]> {
    if polygon.is_empty() {
        return Vec::new();
    }

    let inside = |p: [f64; 3]| -> bool {
        match plane {
            0 => p[0] >= bounds[0],
            1 => p[0] <= bounds[1],
            2 => p[1] >= bounds[2],
            3 => p[1] <= bounds[3],
            4 => p[2] >= bounds[4],
            5 => p[2] <= bounds[5],
            _ => unreachable!(),
        }
    };
    let value = |p: [f64; 3]| -> f64 {
        match plane {
            0 | 1 => p[0],
            2 | 3 => p[1],
            4 | 5 => p[2],
            _ => unreachable!(),
        }
    };
    let limit = bounds[plane];

    let mut out = Vec::new();
    for i in 0..polygon.len() {
        let j = (i + 1) % polygon.len();
        let pi = polygon[i];
        let pj = polygon[j];
        let i_in = inside(pi);
        let j_in = inside(pj);

        if i_in {
            out.push(pi);
        }

        if i_in != j_in {
            let vi = value(pi);
            let vj = value(pj);
            let denom = vj - vi;
            if denom.abs() > 1e-15 {
                let t = ((limit - vi) / denom).clamp(0.0, 1.0);
                out.push([
                    pi[0] + t * (pj[0] - pi[0]),
                    pi[1] + t * (pj[1] - pi[1]),
                    pi[2] + t * (pj[2] - pi[2]),
                ]);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_keeps_inside() {
        let pd = PolyData::from_triangles(
            vec![
                [0.5, 0.5, 0.0],
                [1.0, 0.5, 0.0],
                [0.75, 1.0, 0.0],
                // Outside triangle
                [5.0, 5.0, 0.0],
                [6.0, 5.0, 0.0],
                [5.5, 6.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let result = box_clip(&pd, [0.0, 2.0, 0.0, 2.0, -1.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn clip_splits_intersecting_polygon() {
        let pd = PolyData::from_triangles(
            vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [0.0, 2.0, -1.0, 2.0, -1.0, 1.0]);
        assert!(result.polys.num_cells() > 0);
        for i in 0..result.points.len() {
            assert!(result.points.get(i)[0] >= -1e-12);
        }
    }

    #[test]
    fn clip_removes_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [10.0, 20.0, 10.0, 20.0, 10.0, 20.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn clip_keeps_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = box_clip(&pd, [-10.0, 10.0, -10.0, 10.0, -10.0, 10.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
