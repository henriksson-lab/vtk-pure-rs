use crate::data::{CellArray, Points, PolyData};

/// Plane across which to reflect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflectPlane {
    /// Reflect across X=center plane (negate X)
    X,
    /// Reflect across Y=center plane (negate Y)
    Y,
    /// Reflect across Z=center plane (negate Z)
    Z,
}

/// Parameters for the reflect filter.
pub struct ReflectParams {
    /// The plane to reflect across.
    pub plane: ReflectPlane,
    /// Center value for the reflection plane. Default: 0.0
    pub center: f64,
    /// If true, copy the original mesh and append the reflected copy.
    /// If false, only return the reflected mesh.
    pub copy_input: bool,
}

impl Default for ReflectParams {
    fn default() -> Self {
        Self {
            plane: ReflectPlane::X,
            center: 0.0,
            copy_input: true,
        }
    }
}

/// Reflect a PolyData across a coordinate plane.
///
/// When `copy_input` is true, the result contains both the original and
/// reflected meshes. The reflected polygons have reversed winding order
/// to maintain consistent face normals.
pub fn reflect(input: &PolyData, params: &ReflectParams) -> PolyData {
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut out_lines = CellArray::new();
    let mut out_verts = CellArray::new();
    let mut out_strips = CellArray::new();

    let base_offset = if params.copy_input {
        // Copy original points
        for i in 0..input.points.len() {
            out_points.push(input.points.get(i));
        }
        // Copy original cells
        for cell in input.polys.iter() {
            out_polys.push_cell(cell);
        }
        for cell in input.lines.iter() {
            out_lines.push_cell(cell);
        }
        for cell in input.verts.iter() {
            out_verts.push_cell(cell);
        }
        for cell in input.strips.iter() {
            out_strips.push_cell(cell);
        }
        input.points.len() as i64
    } else {
        0
    };

    // Add reflected points
    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let rp = match params.plane {
            ReflectPlane::X => [2.0 * params.center - p[0], p[1], p[2]],
            ReflectPlane::Y => [p[0], 2.0 * params.center - p[1], p[2]],
            ReflectPlane::Z => [p[0], p[1], 2.0 * params.center - p[2]],
        };
        out_points.push(rp);
    }

    // Add reflected 2D cells with VTK's non-3D reflection ordering.
    for cell in input.polys.iter() {
        let reflected = reflect_non_3d_cell(cell, base_offset);
        out_polys.push_cell(&reflected);
    }
    for cell in input.strips.iter() {
        let reflected = reflect_triangle_strip(cell, base_offset);
        out_strips.push_cell(&reflected);
    }

    // VTK reverses polyline cells but leaves simple line cells unchanged.
    for cell in input.lines.iter() {
        let shifted: Vec<i64> = if cell.len() > 2 {
            cell.iter().rev().map(|&id| id + base_offset).collect()
        } else {
            cell.iter().map(|&id| id + base_offset).collect()
        };
        out_lines.push_cell(&shifted);
    }
    for cell in input.verts.iter() {
        let shifted: Vec<i64> = cell.iter().map(|&id| id + base_offset).collect();
        out_verts.push_cell(&shifted);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd.verts = out_verts;
    pd.strips = out_strips;
    pd
}

fn reflect_non_3d_cell(cell: &[i64], offset: i64) -> Vec<i64> {
    let n = cell.len();
    if n == 0 {
        return Vec::new();
    }

    let mut reflected = vec![0; n];
    for (j, &id) in cell.iter().enumerate() {
        reflected[(n - j) % n] = id + offset;
    }
    reflected
}

fn reflect_triangle_strip(cell: &[i64], offset: i64) -> Vec<i64> {
    if cell.len() % 2 != 0 {
        return reflect_non_3d_cell(cell, offset);
    }

    if cell.len() < 3 {
        return cell.iter().map(|&id| id + offset).collect();
    }

    // VTK inserts a degenerate triangle for strips with an even number of
    // points so every generated triangle is reflected with the correct winding.
    let mut reflected = Vec::with_capacity(cell.len() + 1);
    reflected.push(cell[0] + offset);
    reflected.push(cell[2] + offset);
    reflected.push(cell[1] + offset);
    reflected.push(cell[2] + offset);
    reflected.extend(cell[3..].iter().map(|&id| id + offset));
    reflected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflect_triangle_x() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = reflect(
            &pd,
            &ReflectParams {
                plane: ReflectPlane::X,
                center: 0.0,
                copy_input: true,
            },
        );
        // 3 original + 3 reflected points
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
        // Reflected point 0 should be at (-1.0, 0.0, 0.0)
        let rp = result.points.get(3);
        assert!((rp[0] + 1.0).abs() < 1e-10);
    }

    #[test]
    fn reflect_only_no_copy() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let result = reflect(
            &pd,
            &ReflectParams {
                plane: ReflectPlane::Y,
                center: 5.0,
                copy_input: false,
            },
        );
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        // Y of point 0: 2*5.0 - 2.0 = 8.0
        let rp = result.points.get(0);
        assert!((rp[1] - 8.0).abs() < 1e-10);
    }

    #[test]
    fn reflected_winding_reversed() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = reflect(
            &pd,
            &ReflectParams {
                plane: ReflectPlane::Z,
                center: 0.0,
                copy_input: false,
            },
        );
        // VTK's reflected non-3D cell ordering: [0, 2, 1]
        let cell = result.polys.cell(0);
        assert_eq!(cell, &[0, 2, 1]);
    }

    #[test]
    fn reflects_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2]);

        let result = reflect(
            &pd,
            &ReflectParams {
                plane: ReflectPlane::X,
                center: 0.0,
                copy_input: true,
            },
        );

        assert_eq!(result.strips.num_cells(), 2);
        assert_eq!(result.strips.cell(1), &[3, 5, 4]);
    }

    #[test]
    fn reflects_even_point_strips_with_degenerate_triangle() {
        let mut pd = PolyData::new();
        for p in [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ] {
            pd.points.push(p);
        }
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = reflect(
            &pd,
            &ReflectParams {
                copy_input: false,
                ..Default::default()
            },
        );

        assert_eq!(result.strips.cell(0), &[0, 2, 1, 2, 3]);
    }

    #[test]
    fn reflects_polylines_in_reverse_order() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = reflect(
            &pd,
            &ReflectParams {
                copy_input: false,
                ..Default::default()
            },
        );

        assert_eq!(result.lines.cell(0), &[2, 1, 0]);
    }
}
