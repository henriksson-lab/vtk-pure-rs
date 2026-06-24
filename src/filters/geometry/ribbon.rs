use crate::data::{CellArray, Points, PolyData};

/// Create ribbon (oriented strip) geometry from polyline cells.
///
/// For each line segment, a ribbon of the given `width` is generated
/// perpendicular to the segment direction and a reference `up` vector.
/// The result is a PolyData with triangle strip cells converted to quads.
pub fn ribbon(input: &PolyData, width: f64, up: [f64; 3]) -> PolyData {
    let half = width * 0.5;
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    let up_norm = normalize(up);

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }

        // For each vertex in the polyline, compute the offset direction
        let n = cell.len();
        let mut left_ids = Vec::with_capacity(n);
        let mut right_ids = Vec::with_capacity(n);

        for i in 0..n {
            let p = input.points.get(cell[i] as usize);

            // Tangent direction: average of prev->cur and cur->next
            let tangent = if i == 0 {
                let next = input.points.get(cell[i + 1] as usize);
                normalize(sub(next, p))
            } else if i == n - 1 {
                let prev = input.points.get(cell[i - 1] as usize);
                normalize(sub(p, prev))
            } else {
                let prev = input.points.get(cell[i - 1] as usize);
                let next = input.points.get(cell[i + 1] as usize);
                normalize(sub(next, prev))
            };

            // Side direction = tangent × up, then re-orthogonalize
            let mut side = cross(tangent, up_norm);
            let side_len = length(side);
            if side_len < 1e-10 {
                // Tangent parallel to up, fallback
                side = cross(tangent, [1.0, 0.0, 0.0]);
                if length(side) < 1e-10 {
                    side = cross(tangent, [0.0, 1.0, 0.0]);
                }
            }
            let side = normalize(side);

            let left = [
                p[0] + side[0] * half,
                p[1] + side[1] * half,
                p[2] + side[2] * half,
            ];
            let right = [
                p[0] - side[0] * half,
                p[1] - side[1] * half,
                p[2] - side[2] * half,
            ];

            let base = points.len() as i64;
            points.push(left);
            points.push(right);
            left_ids.push(base);
            right_ids.push(base + 1);
        }

        // Generate quads between consecutive pairs
        for i in 0..n - 1 {
            polys.push_cell(&[left_ids[i], right_ids[i], right_ids[i + 1], left_ids[i + 1]]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = length(v);
    if len < 1e-15 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ribbon_from_line() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = ribbon(&pd, 0.5, [0.0, 0.0, 1.0]);
        // 3 vertices -> 6 ribbon points, 2 quads
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn ribbon_width() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = ribbon(&pd, 1.0, [0.0, 0.0, 1.0]);
        // Width=1 along X-axis with up=Z -> ribbon extends ±0.5 in Y
        let p0 = result.points.get(0);
        let p1 = result.points.get(1);
        assert!((p0[1] - 0.5).abs() < 1e-10 || (p0[1] + 0.5).abs() < 1e-10);
        assert!((p1[1] - 0.5).abs() < 1e-10 || (p1[1] + 0.5).abs() < 1e-10);
        assert!((p0[1] + p1[1]).abs() < 1e-10); // symmetric around Y=0
    }

    #[test]
    fn ribbon_multiple_lines() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[2, 3]);

        let result = ribbon(&pd, 0.2, [0.0, 0.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 2); // 1 quad per line segment
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = ribbon(&pd, 1.0, [0.0, 0.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
