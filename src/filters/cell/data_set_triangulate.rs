use crate::data::{AnyDataArray, DataArray, DataObject, DataSet, UnstructuredGrid};
use crate::types::CellType;

/// Triangulate all cells in an UnstructuredGrid to tetrahedra and triangles.
///
/// Decomposes higher-order 3D cells (hexahedra, wedges, pyramids) into
/// tetrahedra, and 2D cells (quads) into triangles. Already-simplex cells
/// (tetrahedra, triangles) are passed through unchanged.
pub fn data_set_triangulate(input: &UnstructuredGrid) -> UnstructuredGrid {
    let mut output = UnstructuredGrid::new();
    let mut output_cell_sources = Vec::new();

    // Copy all points
    for i in 0..input.num_points() {
        output.points.push(input.point(i));
    }
    *output.point_data_mut() = input.point_data().clone();
    *output.field_data_mut() = input.field_data().clone();

    for ci in 0..input.num_cells() {
        let ct = input.cell_type(ci);
        let pts = input.cell_points(ci);

        match ct {
            CellType::Triangle => {
                push_cell(
                    &mut output,
                    &mut output_cell_sources,
                    ci,
                    CellType::Triangle,
                    pts,
                );
            }
            CellType::Tetra => {
                push_cell(
                    &mut output,
                    &mut output_cell_sources,
                    ci,
                    CellType::Tetra,
                    pts,
                );
            }
            CellType::Pixel => {
                // vtkPixel::TriangulateLocalIds(index = 0)
                if pts.len() >= 4 {
                    const TRIS: [[usize; 3]; 2] = [[0, 1, 3], [0, 3, 2]];
                    push_triangles(&mut output, &mut output_cell_sources, ci, pts, &TRIS);
                }
            }
            CellType::Quad => {
                // vtkQuad::TriangulateLocalIds chooses the shorter diagonal.
                if pts.len() >= 4 {
                    push_quad_triangles(&mut output, &mut output_cell_sources, ci, input, pts);
                }
            }
            CellType::Voxel => {
                // vtkDataSetTriangleFilter uses vtkOrderedTriangulator for 3D cells.
                if pts.len() >= 8 {
                    const TETS: [[usize; 4]; 6] = [
                        [3, 6, 5, 7],
                        [4, 5, 3, 6],
                        [2, 4, 3, 6],
                        [3, 4, 1, 5],
                        [1, 3, 2, 4],
                        [1, 2, 0, 4],
                    ];
                    push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                }
            }
            CellType::Hexahedron => {
                // vtkDataSetTriangleFilter uses vtkOrderedTriangulator for 3D cells.
                if pts.len() >= 8 {
                    const TETS: [[usize; 4]; 6] = [
                        [4, 6, 3, 7],
                        [4, 5, 2, 6],
                        [3, 4, 2, 6],
                        [2, 4, 1, 5],
                        [2, 3, 0, 4],
                        [1, 2, 0, 4],
                    ];
                    push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                }
            }
            CellType::Wedge => {
                // vtkDataSetTriangleFilter flips wedges before ordered triangulation.
                if pts.len() >= 6 {
                    const TETS: [[usize; 4]; 3] = [[2, 4, 3, 5], [1, 3, 2, 4], [0, 2, 1, 3]];
                    push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                }
            }
            CellType::Pyramid => {
                // vtkDataSetTriangleFilter uses vtkOrderedTriangulator for 3D cells.
                if pts.len() >= 5 {
                    const TETS: [[usize; 4]; 2] = [[2, 3, 0, 4], [1, 2, 0, 4]];
                    push_tets(&mut output, &mut output_cell_sources, ci, pts, &TETS);
                }
            }
            CellType::Polygon => {
                // vtkPolygon uses ear cutting, distinct from vtkQuad's triangulation.
                if pts.len() >= 3 {
                    for tri in triangulate_polygon(input, pts) {
                        push_cell(
                            &mut output,
                            &mut output_cell_sources,
                            ci,
                            CellType::Triangle,
                            &tri,
                        );
                    }
                }
            }
            _ => {
                // Pass through unchanged
                push_cell(&mut output, &mut output_cell_sources, ci, ct, pts);
            }
        }
    }

    copy_cell_data(input, &mut output, &output_cell_sources);
    output
}

fn push_cell(
    output: &mut UnstructuredGrid,
    output_cell_sources: &mut Vec<usize>,
    source_cell_id: usize,
    cell_type: CellType,
    point_ids: &[i64],
) {
    output.push_cell(cell_type, point_ids);
    output_cell_sources.push(source_cell_id);
}

fn distance2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn push_quad_triangles(
    output: &mut UnstructuredGrid,
    output_cell_sources: &mut Vec<usize>,
    source_cell_id: usize,
    input: &UnstructuredGrid,
    pts: &[i64],
) {
    let p0 = input.point(pts[0] as usize);
    let p1 = input.point(pts[1] as usize);
    let p2 = input.point(pts[2] as usize);
    let p3 = input.point(pts[3] as usize);
    let d1 = distance2(p0, p2);
    let d2 = distance2(p1, p3);
    if d1 <= d2 {
        const TRIS: [[usize; 3]; 2] = [[0, 1, 2], [0, 2, 3]];
        push_triangles(output, output_cell_sources, source_cell_id, pts, &TRIS);
    } else {
        const TRIS: [[usize; 3]; 2] = [[0, 1, 3], [1, 2, 3]];
        push_triangles(output, output_cell_sources, source_cell_id, pts, &TRIS);
    }
}

fn push_triangles(
    output: &mut UnstructuredGrid,
    output_cell_sources: &mut Vec<usize>,
    source_cell_id: usize,
    pts: &[i64],
    triangles: &[[usize; 3]],
) {
    for triangle in triangles {
        push_cell(
            output,
            output_cell_sources,
            source_cell_id,
            CellType::Triangle,
            &[pts[triangle[0]], pts[triangle[1]], pts[triangle[2]]],
        );
    }
}

fn triangulate_polygon(input: &UnstructuredGrid, pts: &[i64]) -> Vec<[i64; 3]> {
    match pts.len() {
        0..=2 => Vec::new(),
        3 => vec![[pts[0], pts[1], pts[2]]],
        4 => {
            let p: Vec<[f64; 3]> = pts.iter().map(|&id| input.point(id as usize)).collect();
            if simple_polygon_quad_uses_d1(&p) {
                vec![[pts[0], pts[1], pts[2]], [pts[0], pts[2], pts[3]]]
            } else {
                vec![[pts[0], pts[1], pts[3]], [pts[1], pts[2], pts[3]]]
            }
        }
        _ => ear_clip_polygon(input, pts),
    }
}

fn simple_polygon_quad_uses_d1(p: &[[f64; 3]]) -> bool {
    let normal = polygon_normal(p);
    let n2 = dot(normal, normal);
    if n2 == 0.0 {
        return distance2(p[0], p[2]) <= distance2(p[1], p[3]);
    }

    let n012 = cross(sub(p[1], p[0]), sub(p[2], p[0]));
    let n023 = cross(sub(p[2], p[0]), sub(p[3], p[0]));
    let d1_ok = dot(n012, normal) > 0.0 && dot(n023, normal) > 0.0;

    let n013 = cross(sub(p[1], p[0]), sub(p[3], p[0]));
    let n123 = cross(sub(p[2], p[1]), sub(p[3], p[1]));
    let d2_ok = dot(n013, normal) > 0.0 && dot(n123, normal) > 0.0;

    match (d1_ok, d2_ok) {
        (true, false) => true,
        (false, true) => false,
        _ => distance2(p[0], p[2]) < distance2(p[1], p[3]),
    }
}

fn ear_clip_polygon(input: &UnstructuredGrid, pts: &[i64]) -> Vec<[i64; 3]> {
    let coords: Vec<[f64; 3]> = pts.iter().map(|&id| input.point(id as usize)).collect();
    let normal = polygon_normal(&coords);
    let drop_axis = dominant_axis(normal);
    let (u_axis, v_axis) = match drop_axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let coords2d: Vec<[f64; 2]> = coords.iter().map(|p| [p[u_axis], p[v_axis]]).collect();
    let orientation = signed_area2(&coords2d).signum();
    if orientation == 0.0 {
        return fan_triangulate(pts);
    }

    let mut remaining: Vec<usize> = (0..pts.len()).collect();
    let mut result = Vec::new();
    let mut guard = pts.len() * pts.len();
    while remaining.len() > 3 && guard > 0 {
        guard -= 1;
        let mut best = None;
        let mut best_measure = f64::INFINITY;
        let n = remaining.len();
        for i in 0..n {
            let prev = remaining[(i + n - 1) % n];
            let curr = remaining[i];
            let next = remaining[(i + 1) % n];
            if is_polygon_ear(&coords2d, &remaining, prev, curr, next, orientation) {
                let measure = triangle_area2(coords2d[prev], coords2d[curr], coords2d[next]).abs();
                if measure < best_measure {
                    best = Some(i);
                    best_measure = measure;
                }
            }
        }
        let Some(i) = best else {
            return fan_triangulate(pts);
        };
        let n = remaining.len();
        let prev = remaining[(i + n - 1) % n];
        let curr = remaining[i];
        let next = remaining[(i + 1) % n];
        result.push([pts[prev], pts[curr], pts[next]]);
        remaining.remove(i);
    }
    if remaining.len() == 3 {
        result.push([pts[remaining[0]], pts[remaining[1]], pts[remaining[2]]]);
    }
    result
}

fn fan_triangulate(pts: &[i64]) -> Vec<[i64; 3]> {
    (1..pts.len() - 1)
        .map(|i| [pts[0], pts[i], pts[i + 1]])
        .collect()
}

fn is_polygon_ear(
    pts: &[[f64; 2]],
    remaining: &[usize],
    prev: usize,
    curr: usize,
    next: usize,
    orientation: f64,
) -> bool {
    if triangle_area2(pts[prev], pts[curr], pts[next]) * orientation <= 0.0 {
        return false;
    }
    for &idx in remaining {
        if idx != prev
            && idx != curr
            && idx != next
            && point_in_triangle(pts[idx], pts[prev], pts[curr], pts[next])
        {
            return false;
        }
    }
    true
}

fn point_in_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = triangle_area2(p, a, b);
    let d2 = triangle_area2(p, b, c);
    let d3 = triangle_area2(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

fn signed_area2(pts: &[[f64; 2]]) -> f64 {
    let mut area = 0.0;
    for i in 0..pts.len() {
        let j = (i + 1) % pts.len();
        area += pts[i][0] * pts[j][1] - pts[j][0] * pts[i][1];
    }
    area
}

fn triangle_area2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn polygon_normal(pts: &[[f64; 3]]) -> [f64; 3] {
    let mut n = [0.0; 3];
    for i in 0..pts.len() {
        let j = (i + 1) % pts.len();
        n[0] += (pts[i][1] - pts[j][1]) * (pts[i][2] + pts[j][2]);
        n[1] += (pts[i][2] - pts[j][2]) * (pts[i][0] + pts[j][0]);
        n[2] += (pts[i][0] - pts[j][0]) * (pts[i][1] + pts[j][1]);
    }
    n
}

fn dominant_axis(v: [f64; 3]) -> usize {
    if v[0].abs() > v[1].abs() && v[0].abs() > v[2].abs() {
        0
    } else if v[1].abs() > v[2].abs() {
        1
    } else {
        2
    }
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

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn push_tets(
    output: &mut UnstructuredGrid,
    output_cell_sources: &mut Vec<usize>,
    source_cell_id: usize,
    pts: &[i64],
    tets: &[[usize; 4]],
) {
    for tet in tets {
        push_cell(
            output,
            output_cell_sources,
            source_cell_id,
            CellType::Tetra,
            &[pts[tet[0]], pts[tet[1]], pts[tet[2]], pts[tet[3]]],
        );
    }
}

fn copy_cell_data(input: &UnstructuredGrid, output: &mut UnstructuredGrid, source_ids: &[usize]) {
    for array_index in 0..input.cell_data().num_arrays() {
        if let Some(array) = input.cell_data().get_array_by_index(array_index) {
            output
                .cell_data_mut()
                .add_array(copy_tuples(array, source_ids));
        }
    }
}

fn copy_tuples(array: &AnyDataArray, source_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy_variant {
        ($array:expr, $variant:ident) => {{
            let mut copied = DataArray::new($array.name(), $array.num_components());
            for &source_id in source_ids {
                copied.push_tuple($array.tuple(source_id));
            }
            AnyDataArray::$variant(copied)
        }};
    }

    match array {
        AnyDataArray::F32(array) => copy_variant!(array, F32),
        AnyDataArray::F64(array) => copy_variant!(array, F64),
        AnyDataArray::I8(array) => copy_variant!(array, I8),
        AnyDataArray::I16(array) => copy_variant!(array, I16),
        AnyDataArray::I32(array) => copy_variant!(array, I32),
        AnyDataArray::I64(array) => copy_variant!(array, I64),
        AnyDataArray::U8(array) => copy_variant!(array, U8),
        AnyDataArray::U16(array) => copy_variant!(array, U16),
        AnyDataArray::U32(array) => copy_variant!(array, U32),
        AnyDataArray::U64(array) => copy_variant!(array, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulate_hex() {
        let mut grid = UnstructuredGrid::new();
        // Unit cube hex
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([1.0, 1.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.points.push([1.0, 0.0, 1.0]);
        grid.points.push([1.0, 1.0, 1.0]);
        grid.points.push([0.0, 1.0, 1.0]);
        grid.push_cell(CellType::Hexahedron, &[0, 1, 2, 3, 4, 5, 6, 7]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 6); // vtkDataSetTriangleFilter: 1 hex -> 6 tets
        for i in 0..result.num_cells() {
            assert_eq!(result.cell_type(i), CellType::Tetra);
        }
        assert_eq!(result.cell_points(0), &[4, 6, 3, 7]);
        assert_eq!(result.cell_points(5), &[1, 2, 0, 4]);
    }

    #[test]
    fn triangulate_wedge() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.points.push([1.0, 0.0, 1.0]);
        grid.points.push([0.5, 1.0, 1.0]);
        grid.push_cell(CellType::Wedge, &[0, 1, 2, 3, 4, 5]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 3); // 1 wedge -> 3 tets
        assert_eq!(result.cell_points(0), &[2, 4, 3, 5]);
        assert_eq!(result.cell_points(2), &[0, 2, 1, 3]);
    }

    #[test]
    fn triangulate_pyramid() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([1.0, 1.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Pyramid, &[0, 1, 2, 3, 4]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 2); // 1 pyramid -> 2 tets
    }

    #[test]
    fn triangulate_quad_uses_shorter_diagonal() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([2.0, 0.0, 0.0]);
        grid.points.push([2.0, 2.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.push_cell(CellType::Quad, &[0, 1, 2, 3]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 2);
        assert_eq!(result.cell_points(0), &[0, 1, 3]);
        assert_eq!(result.cell_points(1), &[1, 2, 3]);
    }

    #[test]
    fn triangulate_pixel_uses_vtk_index_zero_table() {
        let mut grid = UnstructuredGrid::new();
        for i in 0..4 {
            grid.points.push([i as f64, 0.0, 0.0]);
        }
        grid.push_cell(CellType::Pixel, &[0, 1, 2, 3]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 2);
        assert_eq!(result.cell_points(0), &[0, 1, 3]);
        assert_eq!(result.cell_points(1), &[0, 3, 2]);
    }

    #[test]
    fn triangulate_polygon_quad_uses_vtk_polygon_path() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([1.0, 1.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.push_cell(CellType::Polygon, &[0, 1, 2, 3]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 2);
        assert_eq!(result.cell_points(0), &[0, 1, 3]);
        assert_eq!(result.cell_points(1), &[1, 2, 3]);
    }

    #[test]
    fn triangulate_voxel_uses_vtk_index_zero_table() {
        let mut grid = UnstructuredGrid::new();
        for i in 0..8 {
            grid.points.push([i as f64, 0.0, 0.0]);
        }
        grid.push_cell(CellType::Voxel, &[0, 1, 2, 3, 4, 5, 6, 7]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 6);
        assert_eq!(result.cell_points(0), &[3, 6, 5, 7]);
        assert_eq!(result.cell_points(5), &[1, 2, 0, 4]);
    }

    #[test]
    fn passthrough_tet() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.points.push([0.0, 0.0, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Tetra);
    }

    #[test]
    fn mixed_cells() {
        let mut grid = UnstructuredGrid::new();
        for i in 0..9 {
            grid.points.push([i as f64, 0.0, 0.0]);
        }
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        grid.push_cell(CellType::Quad, &[4, 5, 6, 7]);
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_id",
                (0..9).map(|i| i as f64).collect(),
                1,
            )));
        grid.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "source",
                vec![10.0, 20.0],
                1,
            )));

        let result = data_set_triangulate(&grid);
        assert_eq!(result.num_cells(), 3); // 1 tet + 2 triangles from quad
        assert!(result.point_data().get_array("point_id").is_some());
        let source = result.cell_data().get_array("source").unwrap();
        assert_eq!(source.to_f64_vec(), vec![10.0, 20.0, 20.0]);
    }
}
