use std::collections::{BTreeMap, VecDeque};

use crate::common::core::{Object, VtkIdType};

const EMPTY_EXTENT: [i32; 6] = [0, -1, 0, -1, 0, -1];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExtentSplitterExtent {
    extent: [i32; 6],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExtentSplitterSource {
    extent: [i32; 6],
    priority: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExtentSplitterSubExtent {
    extent: [i32; 6],
    source: i32,
}

#[derive(Debug, Default)]
struct ExtentSplitterInternals {
    sources: BTreeMap<i32, ExtentSplitterSource>,
    queue: VecDeque<ExtentSplitterExtent>,
    sub_extents: Vec<ExtentSplitterSubExtent>,
}

/// VTK: `vtkExtentSplitter`.
#[derive(Debug)]
pub struct ExtentSplitter {
    object: Object,
    internal: ExtentSplitterInternals,
    point_mode: bool,
}

impl ExtentSplitter {
    /// VTK: `vtkExtentSplitter::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkExtentSplitter"),
            internal: ExtentSplitterInternals::default(),
            point_mode: false,
        }
    }

    /// VTK: `vtkExtentSplitter::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.object.get_object_description();
        output.push_str("\nPointMode: ");
        output.push_str(if self.point_mode { "1" } else { "0" });
        if self.internal.sources.is_empty() {
            output.push_str("\nExtent Sources: (none)");
        } else {
            output.push_str("\nExtent Sources: (format = \"id priority: extent\")");
            for (id, source) in &self.internal.sources {
                output.push('\n');
                output.push_str(&format_extent_source(*id, source.priority, source.extent));
            }
        }
        output.push_str("\nNumber of Extents in Queue: ");
        output.push_str(&self.internal.queue.len().to_string());
        if self.internal.sub_extents.is_empty() {
            output.push_str("\nSubExtents: (none)");
        } else {
            output.push_str("\nSubExtents: (format = \"id: extent\")");
            for sub_extent in &self.internal.sub_extents {
                output.push('\n');
                output.push_str(&format_sub_extent(sub_extent.source, sub_extent.extent));
            }
        }
        output
    }

    /// VTK: `vtkExtentSplitter::AddExtentSource`.
    pub fn add_extent_source(
        &mut self,
        id: i32,
        priority: i32,
        x0: i32,
        x1: i32,
        y0: i32,
        y1: i32,
        z0: i32,
        z1: i32,
    ) {
        self.add_extent_source_extent(id, priority, [x0, x1, y0, y1, z0, z1]);
    }

    /// VTK: `vtkExtentSplitter::AddExtentSource(int, int, int*)`.
    pub fn add_extent_source_extent(&mut self, id: i32, priority: i32, extent: [i32; 6]) {
        self.internal
            .sources
            .insert(id, ExtentSplitterSource { extent, priority });
        self.internal.sub_extents.clear();
    }

    /// VTK: `vtkExtentSplitter::RemoveExtentSource`.
    pub fn remove_extent_source(&mut self, id: i32) {
        self.internal.sources.remove(&id);
        self.internal.sub_extents.clear();
    }

    /// VTK: `vtkExtentSplitter::RemoveAllExtentSources`.
    pub fn remove_all_extent_sources(&mut self) {
        self.internal.sources.clear();
        self.internal.sub_extents.clear();
    }

    /// VTK: `vtkExtentSplitter::AddExtent`.
    pub fn add_extent(&mut self, x0: i32, x1: i32, y0: i32, y1: i32, z0: i32, z1: i32) {
        self.add_extent_extent([x0, x1, y0, y1, z0, z1]);
    }

    /// VTK: `vtkExtentSplitter::AddExtent(int*)`.
    pub fn add_extent_extent(&mut self, extent: [i32; 6]) {
        self.internal
            .queue
            .push_back(ExtentSplitterExtent { extent });
        self.internal.sub_extents.clear();
    }

    /// VTK: `vtkExtentSplitter::ComputeSubExtents`.
    pub fn compute_sub_extents(&mut self) -> i32 {
        let mut result = 1;
        let mut sub_extents = Vec::new();
        let mut dimensionality = 0;

        while let Some(mut queued) = self.internal.queue.pop_front() {
            if !self.point_mode {
                dimensionality = extent_dimensionality(queued.extent);
            }

            sub_extents.clear();
            let mut best_priority = -1;
            for (source_id, source) in &self.internal.sources {
                let mut candidate_extent = [0; 6];
                if Self::intersect_extents(queued.extent, source.extent, &mut candidate_extent)
                    && (self.point_mode
                        || dimensionality == extent_dimensionality(candidate_extent))
                {
                    let candidate = ExtentSplitterSubExtent {
                        extent: candidate_extent,
                        source: *source_id,
                    };
                    if source.priority > best_priority {
                        sub_extents.clear();
                        sub_extents.push(candidate);
                        best_priority = source.priority;
                    } else if source.priority == best_priority {
                        sub_extents.push(candidate);
                    }
                }
            }

            if sub_extents.is_empty() {
                result = 0;
                self.internal.sub_extents.push(ExtentSplitterSubExtent {
                    extent: queued.extent,
                    source: -1,
                });
            } else {
                let mut best_volume = 0;
                let mut best_index = 0;
                for (index, candidate) in sub_extents.iter().enumerate() {
                    let volume = extent_volume(candidate.extent);
                    if volume > best_volume {
                        best_volume = volume;
                        best_index = index;
                    }
                }

                let selected = sub_extents[best_index];
                self.internal.sub_extents.push(selected);
                self.split_extent(&mut queued.extent, selected.extent);
            }
        }

        result
    }

    /// VTK: `vtkExtentSplitter::GetNumberOfSubExtents`.
    pub fn get_number_of_sub_extents(&self) -> i32 {
        self.internal.sub_extents.len() as i32
    }

    /// VTK: `vtkExtentSplitter::GetSubExtent`.
    pub fn get_sub_extent(&self, index: i32) -> [i32; 6] {
        let Some(index) = index_to_usize(index) else {
            return EMPTY_EXTENT;
        };
        self.internal
            .sub_extents
            .get(index)
            .map_or(EMPTY_EXTENT, |sub_extent| sub_extent.extent)
    }

    /// VTK: `vtkExtentSplitter::GetSubExtent(int, int*)`.
    pub fn get_sub_extent_into(&self, index: i32, extent: &mut [i32; 6]) {
        *extent = self.get_sub_extent(index);
    }

    /// VTK: `vtkExtentSplitter::GetSubExtentSource`.
    pub fn get_sub_extent_source(&self, index: i32) -> i32 {
        let Some(index) = index_to_usize(index) else {
            return -1;
        };
        self.internal
            .sub_extents
            .get(index)
            .map_or(-1, |sub_extent| sub_extent.source)
    }

    /// VTK: `vtkExtentSplitter::GetPointMode`.
    pub fn get_point_mode(&self) -> bool {
        self.point_mode
    }

    /// VTK: `vtkExtentSplitter::SetPointMode`.
    pub fn set_point_mode(&mut self, point_mode: bool) {
        if self.point_mode != point_mode {
            self.point_mode = point_mode;
            self.modified();
        }
    }

    /// VTK: `vtkExtentSplitter::PointModeOn`.
    pub fn point_mode_on(&mut self) {
        self.set_point_mode(true);
    }

    /// VTK: `vtkExtentSplitter::PointModeOff`.
    pub fn point_mode_off(&mut self) {
        self.set_point_mode(false);
    }

    /// VTK: `vtkExtentSplitter::SplitExtent`.
    pub fn split_extent(&mut self, extent: &mut [i32; 6], subextent: [i32; 6]) {
        let point_mode = if self.point_mode { 1 } else { 0 };

        if extent[4] < subextent[4] {
            self.internal.queue.push_back(ExtentSplitterExtent {
                extent: [
                    extent[0],
                    extent[1],
                    extent[2],
                    extent[3],
                    extent[4],
                    subextent[4] - point_mode,
                ],
            });
            extent[4] = subextent[4];
        }
        if extent[5] > subextent[5] {
            self.internal.queue.push_back(ExtentSplitterExtent {
                extent: [
                    extent[0],
                    extent[1],
                    extent[2],
                    extent[3],
                    subextent[5] + point_mode,
                    extent[5],
                ],
            });
            extent[5] = subextent[5];
        }

        if extent[2] < subextent[2] {
            self.internal.queue.push_back(ExtentSplitterExtent {
                extent: [
                    extent[0],
                    extent[1],
                    extent[2],
                    subextent[2] - point_mode,
                    extent[4],
                    extent[5],
                ],
            });
            extent[2] = subextent[2];
        }
        if extent[3] > subextent[3] {
            self.internal.queue.push_back(ExtentSplitterExtent {
                extent: [
                    extent[0],
                    extent[1],
                    subextent[3] + point_mode,
                    extent[3],
                    extent[4],
                    extent[5],
                ],
            });
            extent[3] = subextent[3];
        }

        if extent[0] < subextent[0] {
            self.internal.queue.push_back(ExtentSplitterExtent {
                extent: [
                    extent[0],
                    subextent[0] - point_mode,
                    extent[2],
                    extent[3],
                    extent[4],
                    extent[5],
                ],
            });
            extent[0] = subextent[0];
        }
        if extent[1] > subextent[1] {
            self.internal.queue.push_back(ExtentSplitterExtent {
                extent: [
                    subextent[1] + point_mode,
                    extent[1],
                    extent[2],
                    extent[3],
                    extent[4],
                    extent[5],
                ],
            });
        }
    }

    /// VTK: `vtkExtentSplitter::IntersectExtents`.
    pub fn intersect_extents(extent1: [i32; 6], extent2: [i32; 6], result: &mut [i32; 6]) -> bool {
        if extent1[0] > extent2[1]
            || extent1[2] > extent2[3]
            || extent1[4] > extent2[5]
            || extent1[1] < extent2[0]
            || extent1[3] < extent2[2]
            || extent1[5] < extent2[4]
        {
            return false;
        }

        result[0] = extent1[0].max(extent2[0]);
        result[1] = extent1[1].min(extent2[1]);
        result[2] = extent1[2].max(extent2[2]);
        result[3] = extent1[3].min(extent2[3]);
        result[4] = extent1[4].max(extent2[4]);
        result[5] = extent1[5].min(extent2[5]);
        true
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkExtentSplitter::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkExtentSplitter" || Object::is_type_of(name)
    }

    /// VTK: `vtkExtentSplitter::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkExtentSplitter::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkExtentSplitter" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkExtentSplitter::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObject::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObject::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObject::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.object.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.object.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.object.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for ExtentSplitter {
    fn default() -> Self {
        Self::new()
    }
}

fn index_to_usize(index: i32) -> Option<usize> {
    usize::try_from(index).ok()
}

fn extent_dimensionality(extent: [i32; 6]) -> i32 {
    ((extent[1] - extent[0] > 0) as i32)
        + ((extent[3] - extent[2] > 0) as i32)
        + ((extent[5] - extent[4] > 0) as i32)
}

fn extent_volume(extent: [i32; 6]) -> i32 {
    (extent[1] - extent[0] + 1) * (extent[3] - extent[2] + 1) * (extent[5] - extent[4] + 1)
}

fn format_extent_source(id: i32, priority: i32, extent: [i32; 6]) -> String {
    format!(
        "{} {}: {} {}  {} {}  {} {}",
        id, priority, extent[0], extent[1], extent[2], extent[3], extent[4], extent[5]
    )
}

fn format_sub_extent(source: i32, extent: [i32; 6]) -> String {
    format!(
        "{}: {} {}  {} {}  {} {}",
        source, extent[0], extent[1], extent[2], extent[3], extent[4], extent[5]
    )
}
