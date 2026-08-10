use crate::common::core::{Object, VtkIdType, VtkMTimeType};

/// VTK: `vtkGraphEdge`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    object: Object,
    source: VtkIdType,
    target: VtkIdType,
    id: VtkIdType,
}

impl GraphEdge {
    /// VTK: `vtkGraphEdge::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkGraphEdge"),
            source: 0,
            target: 0,
            id: 0,
        }
    }

    /// VTK: `vtkGraphEdge::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nSource: {}\nTarget: {}\nId: {}",
            self.object.get_object_description(),
            self.source,
            self.target,
            self.id
        )
    }

    /// VTK: `vtkGraphEdge::SetSource`.
    pub fn set_source(&mut self, source: VtkIdType) {
        if self.source != source {
            self.source = source;
            self.modified();
        }
    }

    /// VTK: `vtkGraphEdge::GetSource`.
    pub fn get_source(&self) -> VtkIdType {
        self.source
    }

    /// VTK: `vtkGraphEdge::SetTarget`.
    pub fn set_target(&mut self, target: VtkIdType) {
        if self.target != target {
            self.target = target;
            self.modified();
        }
    }

    /// VTK: `vtkGraphEdge::GetTarget`.
    pub fn get_target(&self) -> VtkIdType {
        self.target
    }

    /// VTK: `vtkGraphEdge::SetId`.
    pub fn set_id(&mut self, id: VtkIdType) {
        if self.id != id {
            self.id = id;
            self.modified();
        }
    }

    /// VTK: `vtkGraphEdge::GetId`.
    pub fn get_id(&self) -> VtkIdType {
        self.id
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl Default for GraphEdge {
    fn default() -> Self {
        Self::new()
    }
}
