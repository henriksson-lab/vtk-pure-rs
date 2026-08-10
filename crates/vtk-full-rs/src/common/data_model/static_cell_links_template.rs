use std::{fmt::Debug, sync::Arc};

use crate::common::core::{IdList, VtkIdType};

use super::{CellArray, DataSetApi};

/// Integer id type usable by `vtkStaticCellLinksTemplate`.
pub trait StaticCellLinkId:
    Copy + Debug + Default + Ord + TryFrom<VtkIdType> + Into<VtkIdType> + Send + Sync + 'static
{
}

impl<T> StaticCellLinkId for T where
    T: Copy + Debug + Default + Ord + TryFrom<VtkIdType> + Into<VtkIdType> + Send + Sync + 'static
{
}

/// VTK: `vtkStaticCellLinksTemplate<TIds>`.
#[derive(Clone, Debug)]
pub struct StaticCellLinksTemplate<TIds: StaticCellLinkId> {
    links_size: TIds,
    num_pts: TIds,
    num_cells: TIds,
    links: Option<Arc<Vec<TIds>>>,
    offsets: Option<Arc<Vec<TIds>>>,
}

impl<TIds: StaticCellLinkId> StaticCellLinksTemplate<TIds> {
    /// VTK: `vtkStaticCellLinksTemplate::vtkStaticCellLinksTemplate`.
    pub fn new() -> Self {
        Self {
            links_size: TIds::default(),
            num_pts: TIds::default(),
            num_cells: TIds::default(),
            links: None,
            offsets: None,
        }
    }

    /// VTK: `vtkStaticCellLinksTemplate::Initialize`.
    pub fn initialize(&mut self) {
        self.links_size = TIds::default();
        self.num_pts = TIds::default();
        self.num_cells = TIds::default();
        self.links = None;
        self.offsets = None;
    }

    /// VTK: `vtkStaticCellLinksTemplate::BuildLinks(vtkDataSet*)`.
    pub fn build_links(&mut self, data_set: &dyn DataSetApi) {
        self.initialize();
        let num_pts = data_set.get_number_of_points();
        let num_cells = data_set.get_number_of_cells();
        let mut cells = Vec::with_capacity(vtk_id_to_usize(num_cells));
        let mut cell_points = IdList::new();
        for cell_id in 0..num_cells {
            data_set.get_cell_points(cell_id, &mut cell_points);
            cells.push(cell_points.iter().collect::<Vec<_>>());
        }
        self.build_from_cell_slices(num_pts, num_cells, cells.iter().map(Vec::as_slice));
    }

    /// VTK: `vtkStaticCellLinksTemplate::BuildLinks(vtkIdType, vtkIdType, vtkCellArray*)`.
    pub fn build_links_from_cell_array(
        &mut self,
        num_pts: VtkIdType,
        num_cells: VtkIdType,
        cell_array: &CellArray,
    ) {
        self.build_links_from_multiple_arrays(num_pts, num_cells, [cell_array]);
    }

    /// VTK: `vtkStaticCellLinksTemplate::BuildLinksFromMultipleArrays`.
    pub fn build_links_from_multiple_arrays<'a, I>(
        &mut self,
        num_pts: VtkIdType,
        num_cells: VtkIdType,
        cell_arrays: I,
    ) where
        I: IntoIterator<Item = &'a CellArray>,
    {
        let cell_arrays = cell_arrays.into_iter().collect::<Vec<_>>();
        let cell_id_offsets = cell_arrays
            .iter()
            .scan(0, |offset, cell_array| {
                let current = *offset;
                *offset += cell_array.get_number_of_cells();
                Some(current)
            })
            .collect::<Vec<_>>();
        let cells = cell_arrays
            .iter()
            .zip(cell_id_offsets.iter())
            .flat_map(|(cell_array, id_offset)| {
                cell_array
                    .iter()
                    .enumerate()
                    .map(move |(cell_id, point_ids)| (*id_offset + cell_id as VtkIdType, point_ids))
            })
            .collect::<Vec<_>>();
        self.build_from_indexed_cell_slices(num_pts, num_cells, cells.into_iter());
    }

    /// VTK: `vtkStaticCellLinksTemplate::GetNumberOfCells`.
    pub fn get_number_of_cells(&self, pt_id: VtkIdType) -> TIds {
        let offsets = self.offsets();
        let pt_id = vtk_id_to_usize(pt_id);
        link_id(offsets[pt_id + 1].into() - offsets[pt_id].into())
    }

    /// VTK: `vtkStaticCellLinksTemplate::GetNcells`.
    pub fn get_ncells(&self, pt_id: VtkIdType) -> VtkIdType {
        self.get_number_of_cells(pt_id).into()
    }

    /// VTK: `vtkStaticCellLinksTemplate::GetCells(vtkIdType)`.
    pub fn get_cells(&self, pt_id: VtkIdType) -> &[TIds] {
        let offsets = self.offsets();
        let links = self.links();
        let pt_id = vtk_id_to_usize(pt_id);
        let start = vtk_id_to_usize(offsets[pt_id].into());
        let end = vtk_id_to_usize(offsets[pt_id + 1].into());
        &links[start..end]
    }

    /// VTK: `vtkStaticCellLinksTemplate::MatchesCell`.
    pub fn matches_cell(&self, point_ids: &[VtkIdType]) -> bool {
        self.cells_matching_points(point_ids).next().is_some()
    }

    /// VTK: `vtkStaticCellLinksTemplate::GetCells(vtkIdType, const vtkIdType*, vtkIdList*)`.
    pub fn get_cells_for_points(&self, point_ids: &[VtkIdType], cells: &mut IdList) {
        cells.reset();
        for cell_id in self.cells_matching_points(point_ids) {
            cells.insert_next_id(cell_id);
        }
    }

    /// VTK: `vtkStaticCellLinksTemplate::GetLinksSize`.
    pub fn get_links_size(&self) -> TIds {
        self.links_size
    }

    /// VTK: `vtkStaticCellLinksTemplate::GetOffset`.
    pub fn get_offset(&self, pt_id: VtkIdType) -> TIds {
        self.offsets()[vtk_id_to_usize(pt_id)]
    }

    /// VTK: `vtkStaticCellLinksTemplate::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> u64 {
        if self.links.is_none() {
            return 0;
        }
        let bytes = (self.links_size.into() + 1) as u64 * size_of::<TIds>() as u64
            + (self.num_pts.into() + 1) as u64 * size_of::<TIds>() as u64;
        bytes.div_ceil(1024)
    }

    /// VTK: `vtkStaticCellLinksTemplate::DeepCopy`.
    pub fn deep_copy(&mut self, links: &Self) {
        self.links_size = links.links_size;
        self.num_pts = links.num_pts;
        self.num_cells = links.num_cells;
        self.links = links
            .links
            .as_ref()
            .map(|values| Arc::new((**values).clone()));
        self.offsets = links
            .offsets
            .as_ref()
            .map(|values| Arc::new((**values).clone()));
    }

    /// VTK: `vtkStaticCellLinksTemplate::ShallowCopy`.
    pub fn shallow_copy(&mut self, links: &Self) {
        self.links_size = links.links_size;
        self.num_pts = links.num_pts;
        self.num_cells = links.num_cells;
        self.links = links.links.as_ref().map(Arc::clone);
        self.offsets = links.offsets.as_ref().map(Arc::clone);
    }

    /// VTK: `vtkStaticCellLinksTemplate::SelectCells`.
    pub fn select_cells(&self, min_max_degree: [VtkIdType; 2], cell_selection: &mut [u8]) {
        for value in cell_selection
            .iter_mut()
            .take(self.num_cells.into() as usize)
        {
            *value = 0;
        }
        for pt_id in 0..self.num_pts.into() {
            let degree = self.get_ncells(pt_id);
            if degree >= min_max_degree[0] && degree < min_max_degree[1] {
                for &cell_id in self.get_cells(pt_id) {
                    let cell_id = vtk_id_to_usize(cell_id.into());
                    if let Some(selection) = cell_selection.get_mut(cell_id) {
                        *selection = 1;
                    }
                }
            }
        }
    }

    fn build_from_cell_slices<'a, I>(&mut self, num_pts: VtkIdType, num_cells: VtkIdType, cells: I)
    where
        I: IntoIterator<Item = &'a [VtkIdType]>,
    {
        self.build_from_indexed_cell_slices(
            num_pts,
            num_cells,
            cells
                .into_iter()
                .enumerate()
                .map(|(cell_id, point_ids)| (cell_id as VtkIdType, point_ids)),
        );
    }

    fn build_from_indexed_cell_slices<'a, I>(
        &mut self,
        num_pts: VtkIdType,
        num_cells: VtkIdType,
        cells: I,
    ) where
        I: IntoIterator<Item = (VtkIdType, &'a [VtkIdType])>,
    {
        self.initialize();
        let cells = cells.into_iter().collect::<Vec<_>>();
        let mut counts = vec![0; vtk_id_to_usize(num_pts)];
        let mut links_size = 0;
        for (_, point_ids) in &cells {
            for &pt_id in *point_ids {
                counts[vtk_id_to_usize(pt_id)] += 1;
                links_size += 1;
            }
        }

        let mut offsets = vec![TIds::default(); vtk_id_to_usize(num_pts) + 1];
        for pt_id in 1..vtk_id_to_usize(num_pts) {
            offsets[pt_id] = link_id(offsets[pt_id - 1].into() + counts[pt_id - 1]);
        }
        offsets[vtk_id_to_usize(num_pts)] = link_id(links_size);

        let mut remaining = counts;
        let mut links = vec![TIds::default(); vtk_id_to_usize(links_size) + 1];
        links[vtk_id_to_usize(links_size)] = link_id(num_pts);
        for (cell_id, point_ids) in cells {
            for &pt_id in point_ids {
                let idx = vtk_id_to_usize(pt_id);
                let offset = offsets[idx + 1].into() - remaining[idx];
                remaining[idx] -= 1;
                links[vtk_id_to_usize(offset)] = link_id(cell_id);
            }
        }

        for pt_id in 0..vtk_id_to_usize(num_pts) {
            let start = vtk_id_to_usize(offsets[pt_id].into());
            let end = vtk_id_to_usize(offsets[pt_id + 1].into());
            links[start..end].sort();
        }

        self.links_size = link_id(links_size);
        self.num_pts = link_id(num_pts);
        self.num_cells = link_id(num_cells);
        self.links = Some(Arc::new(links));
        self.offsets = Some(Arc::new(offsets));
    }

    fn cells_matching_points<'a>(
        &'a self,
        point_ids: &'a [VtkIdType],
    ) -> impl Iterator<Item = VtkIdType> + 'a {
        let min_point = point_ids
            .iter()
            .copied()
            .min_by_key(|&pt_id| self.get_ncells(pt_id));
        min_point.into_iter().flat_map(move |pt_id| {
            self.get_cells(pt_id)
                .iter()
                .copied()
                .map(Into::into)
                .filter(move |&cell_id| {
                    point_ids.iter().all(|&other_pt_id| {
                        other_pt_id == pt_id
                            || self
                                .get_cells(other_pt_id)
                                .iter()
                                .any(|&linked_cell| linked_cell.into() == cell_id)
                    })
                })
        })
    }

    fn links(&self) -> &[TIds] {
        self.links.as_deref().map(Vec::as_slice).unwrap_or(&[])
    }

    fn offsets(&self) -> &[TIds] {
        self.offsets.as_deref().map(Vec::as_slice).unwrap_or(&[])
    }
}

impl<TIds: StaticCellLinkId> Default for StaticCellLinksTemplate<TIds> {
    fn default() -> Self {
        Self::new()
    }
}

fn link_id<TIds: StaticCellLinkId>(value: VtkIdType) -> TIds {
    TIds::try_from(value).ok().expect("link id out of range")
}

fn vtk_id_to_usize(value: VtkIdType) -> usize {
    usize::try_from(value).expect("vtk id must be non-negative")
}
