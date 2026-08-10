use crate::data::{
    AnyDataArray, CellArray, DataArray, DataSetAttributes, KdTree, Points, PolyData,
};
use crate::types::Scalar;

/// Merge vertices that are within `distance` of each other using k-d tree.
///
/// Rust counterpart of `vtkCleanPolyData` (`Filters/Core/vtkCleanPolyData.cxx`)
/// running with `PointMerging` on and `ToleranceIsAbsolute` on, i.e. `distance`
/// is an absolute tolerance (VTK's `AbsoluteTolerance`), not a fraction of the
/// bounding box diagonal. Like VTK's locator, a point merges into the first
/// already-inserted point within the tolerance (`<=`, see
/// `vtkPointLocator::IsInsertedPoint`); the k-d tree only accelerates that
/// search.
///
/// Connectivity is rewritten the way `vtkCleanPolyData` does it: consecutive
/// duplicate point ids are collapsed, a polygon/strip whose first and last id
/// coincide loses the repeat, and cells that degenerate are downgraded
/// (strip -> poly -> line -> vert), matching VTK's default `ConvertStripsToPolys`
/// / `ConvertPolysToLines` / `ConvertLinesToPoints` settings.
///
/// Deviation from VTK: points that no cell references are kept (VTK only emits
/// points it meets while walking cells); use `remove_isolated_vertices` for that.
pub fn merge_close_vertices(input: &PolyData, distance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 || !distance.is_finite() || distance < 0.0 {
        return input.clone();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);
    let d2 = distance * distance;

    let mut remap = vec![usize::MAX; n];
    let mut out_pts = Points::<f64>::new();
    let mut representatives = Vec::new();

    for i in 0..n {
        if remap[i] != usize::MAX {
            continue;
        }
        let idx = out_pts.len();
        out_pts.push(pts[i]);
        representatives.push(i);
        remap[i] = idx;

        let nbrs = tree.find_within_radius(pts[i], distance);
        for &(j, jd2) in &nbrs {
            if j != i && remap[j] == usize::MAX && jd2 <= d2 {
                remap[j] = idx;
            }
        }
    }

    // Input cells are numbered verts, lines, polys, strips - the order
    // vtkPolyData uses for cell data.
    let mut out_cells = OutputCells::default();
    let mut old_cell_id = 0usize;
    for (cells, kind) in [
        (&input.verts, CellKind::Verts),
        (&input.lines, CellKind::Lines),
        (&input.polys, CellKind::Polys),
        (&input.strips, CellKind::Strips),
    ] {
        for cell in cells.iter() {
            let cell_id = old_cell_id;
            old_cell_id += 1;
            let Some(mapped) = remap_cell(cell, &remap) else {
                continue;
            };
            let cleaned = collapse_repeated_ids(&mapped, kind);
            let Some(target) = downgrade(kind, cleaned.len()) else {
                continue;
            };
            out_cells.push(target, cleaned, cell_id);
        }
    }
    let (out_verts, out_lines, out_polys, out_strips, kept_cell_ids) = out_cells.finish();

    let mut pd = input.clone();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd.strips = out_strips;
    remap_point_data(input, &representatives, &mut pd);
    remap_cell_data(input, &kept_cell_ids, &mut pd);
    pd
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Verts,
    Lines,
    Polys,
    Strips,
}

/// Collapse runs of identical point ids, as vtkCleanPolyData does while
/// rewriting connectivity (`if (i == 0 || ptId != updatedPts[numNewPts - 1])`).
/// Vertex cells keep every id; polygons and strips additionally drop a trailing
/// id that repeats the first one.
fn collapse_repeated_ids(mapped: &[i64], kind: CellKind) -> Vec<i64> {
    if kind == CellKind::Verts {
        return mapped.to_vec();
    }
    let mut out: Vec<i64> = Vec::with_capacity(mapped.len());
    for &id in mapped {
        if out.last() != Some(&id) {
            out.push(id);
        }
    }
    match kind {
        CellKind::Polys if out.len() > 2 && out.first() == out.last() => {
            out.pop();
        }
        CellKind::Strips if out.len() > 1 && out.first() == out.last() => {
            out.pop();
        }
        _ => {}
    }
    out
}

/// Which cell array a cleaned cell ends up in. Mirrors vtkCleanPolyData with
/// ConvertStripsToPolys/ConvertPolysToLines/ConvertLinesToPoints all enabled
/// (their default): a cell too small for its own array is demoted rather than
/// dropped, and only an empty cell disappears.
fn downgrade(kind: CellKind, num_points: usize) -> Option<CellKind> {
    let target = match (kind, num_points) {
        (_, 0) => return None,
        (CellKind::Strips, n) if n > 3 => CellKind::Strips,
        (CellKind::Polys | CellKind::Strips, n) if n > 2 => CellKind::Polys,
        (CellKind::Lines | CellKind::Polys | CellKind::Strips, n) if n > 1 => CellKind::Lines,
        (CellKind::Verts, _) => CellKind::Verts,
        _ => CellKind::Verts,
    };
    Some(target)
}

/// Cells being accumulated per output array, each remembering the input cell it
/// came from so cell data can follow. vtkCleanPolyData does the same with its
/// separate outLineData/outPolyData/outStrpData lists.
#[derive(Default)]
struct OutputCells {
    verts: Vec<(Vec<i64>, usize)>,
    lines: Vec<(Vec<i64>, usize)>,
    polys: Vec<(Vec<i64>, usize)>,
    strips: Vec<(Vec<i64>, usize)>,
}

impl OutputCells {
    fn push(&mut self, kind: CellKind, cell: Vec<i64>, old_cell_id: usize) {
        let bucket = match kind {
            CellKind::Verts => &mut self.verts,
            CellKind::Lines => &mut self.lines,
            CellKind::Polys => &mut self.polys,
            CellKind::Strips => &mut self.strips,
        };
        bucket.push((cell, old_cell_id));
    }

    fn finish(self) -> (CellArray, CellArray, CellArray, CellArray, Vec<usize>) {
        let mut kept_cell_ids = Vec::new();
        let mut build = |cells: Vec<(Vec<i64>, usize)>| {
            let mut out = CellArray::new();
            for (cell, old_cell_id) in cells {
                out.push_cell(&cell);
                kept_cell_ids.push(old_cell_id);
            }
            out
        };
        let verts = build(self.verts);
        let lines = build(self.lines);
        let polys = build(self.polys);
        let strips = build(self.strips);
        (verts, lines, polys, strips, kept_cell_ids)
    }
}

fn remap_cell(cell: &[i64], remap: &[usize]) -> Option<Vec<i64>> {
    let mut mapped = Vec::with_capacity(cell.len());
    for &id in cell {
        if id < 0 || id as usize >= remap.len() {
            return None;
        }
        mapped.push(remap[id as usize] as i64);
    }
    Some(mapped)
}

fn remap_point_data(input: &PolyData, old_point_ids: &[usize], output: &mut PolyData) {
    output.point_data_mut().clear();
    for array in input.point_data().iter() {
        if array.num_tuples() == input.points.len() {
            output
                .point_data_mut()
                .add_array(select_tuples(array, old_point_ids));
        }
    }
    copy_active_attributes(input.point_data(), output.point_data_mut());
}

fn remap_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    output.cell_data_mut().clear();
    for array in input.cell_data().iter() {
        if array.num_tuples() == input.total_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, old_cell_ids));
        }
    }
    copy_active_attributes(input.cell_data(), output.cell_data_mut());
}

fn select_tuples(array: &AnyDataArray, old_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(select_typed_tuples($array, old_ids))
        };
    }

    match array {
        AnyDataArray::F32(array) => select!(array, F32),
        AnyDataArray::F64(array) => select!(array, F64),
        AnyDataArray::I8(array) => select!(array, I8),
        AnyDataArray::I16(array) => select!(array, I16),
        AnyDataArray::I32(array) => select!(array, I32),
        AnyDataArray::I64(array) => select!(array, I64),
        AnyDataArray::U8(array) => select!(array, U8),
        AnyDataArray::U16(array) => select!(array, U16),
        AnyDataArray::U32(array) => select!(array, U32),
        AnyDataArray::U64(array) => select!(array, U64),
    }
}

fn select_typed_tuples<T: Scalar>(array: &DataArray<T>, old_ids: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(old_ids.len() * num_components);
    for &old_id in old_ids {
        data.extend_from_slice(array.tuple(old_id));
    }
    DataArray::from_vec(array.name(), data, num_components)
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]); // close to 0
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.001, 0.0, 0.0]); // close to 2
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 2, 4]);
        pd.polys.push_cell(&[1, 3, 4]);

        let result = merge_close_vertices(&pd, 0.01);
        assert!(result.points.len() < 5);
    }

    #[test]
    fn no_merge_far() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = merge_close_vertices(&pd, 0.001);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn preserves_representative_point_and_cell_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.001, 0.0, 0.0]);
        pd.lines.push_cell(&[4, 5]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temperature",
                vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
                1,
            )));
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("ids", vec![7, 9], 1)));

        let result = merge_close_vertices(&pd, 0.01);

        let temperature = result.point_data().get_array("temperature").unwrap();
        let mut scalar = [0.0f64];
        temperature.tuple_as_f64(0, &mut scalar);
        assert_eq!(scalar[0], 10.0);

        // The line collapsed to a single point, so vtkCleanPolyData emits it as
        // a vertex cell; cell data is reordered verts-then-polys accordingly.
        assert_eq!(result.verts.num_cells(), 1);
        assert_eq!(result.lines.num_cells(), 0);
        let ids = result.cell_data().get_array("ids").unwrap();
        assert_eq!(ids.num_tuples(), 2);
        ids.tuple_as_f64(0, &mut scalar);
        assert_eq!(scalar[0], 7.0);
        ids.tuple_as_f64(1, &mut scalar);
        assert_eq!(scalar[0], 9.0);
    }

    #[test]
    fn collapsed_polygon_is_downgraded_to_a_line() {
        // vtkCleanPolyData with ConvertPolysToLines on turns a triangle whose
        // two first corners merge into a line rather than dropping it.
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = merge_close_vertices(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.lines.cell(0), &[0, 1]);
    }

    #[test]
    fn collapsed_strip_is_downgraded_to_a_polygon() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2, 3]);

        let result = merge_close_vertices(&pd, 0.01);
        assert_eq!(result.strips.num_cells(), 0);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn quadrilateral_keeps_its_remaining_corners() {
        // Merging two adjacent corners of a quad leaves a triangle, matching
        // vtkCleanPolyData's consecutive-duplicate collapse.
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = merge_close_vertices(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn keeps_active_scalars_and_line_topology() {
        let mut mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        mesh.lines.push_cell(&[0, 2]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![10.0, 11.0, 12.0, 13.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("id");

        let r = merge_close_vertices(&mesh, 0.01);
        assert_eq!(r.points.len(), 3);
        assert_eq!(r.lines.num_cells(), 1);

        let arr = r.point_data().get_array("id").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut value = [0.0];
        arr.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 10.0);
        assert!(r.point_data().scalars().is_some());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(merge_close_vertices(&pd, 0.1).points.len(), 0);
    }

    #[test]
    fn invalid_distance_is_noop() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 1]);

        assert_eq!(merge_close_vertices(&pd, -1.0).points.len(), 2);
        assert_eq!(merge_close_vertices(&pd, f64::NAN).points.len(), 2);
    }

    #[test]
    fn skips_cells_with_invalid_point_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = merge_close_vertices(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
