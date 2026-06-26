use crate::data::{CellArray, Points, PolyData};

/// Triangulate closed 2D contours (polylines) using ear-clipping.
///
/// Input: PolyData with closed line loops (each line cell is treated as a closed polygon).
/// Output: PolyData with triangle polygons.
///
/// Assumes the contour lies approximately in a plane. Uses the XY projection
/// for the ear-clipping test.
pub fn contour_triangulator(input: &PolyData) -> PolyData {
    if input.lines.num_cells() == 0 {
        return PolyData::new();
    }

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    // Copy all points
    for i in 0..input.points.len() {
        out_points.push(input.points.get(i));
    }

    // Process each line cell as a closed contour
    for cell in input.lines.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Remove duplicate last vertex if the contour is explicitly closed
        let mut indices: Vec<usize> = cell.iter().map(|&id| id as usize).collect();
        if indices.len() > 1 && indices[0] == indices[indices.len() - 1] {
            indices.pop();
        }
        if indices.len() < 3 {
            continue;
        }

        ear_clip(&out_points, &mut indices, &mut out_polys);
    }

    let mut result = PolyData::new();
    result.points = out_points;
    result.polys = out_polys;
    *result.point_data_mut() = input.point_data().clone();
    result
}

fn ear_clip(points: &Points<f64>, indices: &mut Vec<usize>, polys: &mut CellArray) {
    // Ensure polygon is counter-clockwise in XY
    if signed_area_2d(points, indices) < 0.0 {
        indices.reverse();
    }

    let mut remaining = indices.clone();

    while remaining.len() > 3 {
        let n = remaining.len();
        let mut found_ear = false;

        for i in 0..n {
            let prev = remaining[(i + n - 1) % n];
            let curr = remaining[i];
            let next = remaining[(i + 1) % n];

            if !is_convex_2d(points, prev, curr, next) {
                continue;
            }

            // Check that no other vertex is inside this triangle
            let mut is_ear = true;
            for j in 0..n {
                if j == (i + n - 1) % n || j == i || j == (i + 1) % n {
                    continue;
                }
                if point_in_triangle_2d(points, prev, curr, next, remaining[j]) {
                    is_ear = false;
                    break;
                }
            }

            if is_ear {
                polys.push_cell(&[prev as i64, curr as i64, next as i64]);
                remaining.remove(i);
                found_ear = true;
                break;
            }
        }

        if !found_ear {
            // Degenerate polygon; emit remaining as a fan to avoid infinite loop
            for i in 1..remaining.len() - 1 {
                polys.push_cell(&[
                    remaining[0] as i64,
                    remaining[i] as i64,
                    remaining[i + 1] as i64,
                ]);
            }
            return;
        }
    }

    if remaining.len() == 3 {
        polys.push_cell(&[
            remaining[0] as i64,
            remaining[1] as i64,
            remaining[2] as i64,
        ]);
    }
}

fn signed_area_2d(points: &Points<f64>, indices: &[usize]) -> f64 {
    let n = indices.len();
    let mut area = 0.0;
    for i in 0..n {
        let a = points.get(indices[i]);
        let b = points.get(indices[(i + 1) % n]);
        area += a[0] * b[1] - b[0] * a[1];
    }
    area * 0.5
}

fn is_convex_2d(points: &Points<f64>, a: usize, b: usize, c: usize) -> bool {
    let pa = points.get(a);
    let pb = points.get(b);
    let pc = points.get(c);
    let cross = (pb[0] - pa[0]) * (pc[1] - pa[1]) - (pb[1] - pa[1]) * (pc[0] - pa[0]);
    cross > 0.0
}

fn point_in_triangle_2d(points: &Points<f64>, a: usize, b: usize, c: usize, p: usize) -> bool {
    let pa = points.get(a);
    let pb = points.get(b);
    let pc = points.get(c);
    let pp = points.get(p);

    let d1 = sign(pp, pa, pb);
    let d2 = sign(pp, pb, pc);
    let d3 = sign(pp, pc, pa);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

fn sign(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3]) -> f64 {
    (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulate_square_contour() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);

        // Closed line loop (4 vertices)
        pd.lines.push_cell(&[0, 1, 2, 3]);

        let result = contour_triangulator(&pd);
        assert_eq!(result.polys.num_cells(), 2); // Square -> 2 triangles
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn triangulate_triangle_contour() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);

        pd.lines.push_cell(&[0, 1, 2]);

        let result = contour_triangulator(&pd);
        assert_eq!(result.polys.num_cells(), 1); // Already a triangle
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = contour_triangulator(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
