use std::{
    collections::{HashMap, HashSet},
    ffi::c_void,
    sync::{Mutex, MutexGuard, OnceLock},
};

use crate::common::core::{
    object::Object,
    string_token::{StringToken, StringTokenHash},
    vtk_type::{VtkIdType, VtkMTimeType},
};

use super::CellGridQuery;

/// VTK: `vtkCellGrid*`.
pub type CellGridMetadataHandle = *mut c_void;

/// VTK: `vtkCellMetadata::CellTypeId`.
pub type CellTypeId = StringTokenHash;

/// VTK: `vtkCellMetadata::DOFType`.
pub type DofType = StringToken;

/// VTK: `vtkCellMetadata::MetadataConstructor`.
pub type MetadataConstructor = fn(CellGridMetadataHandle) -> CellMetadata;

type ConstructorMap = HashMap<StringToken, MetadataConstructor>;
type ResponderMap =
    HashMap<StringToken, HashMap<StringToken, Box<dyn CellGridResponderApi + Send>>>;

fn constructors() -> &'static Mutex<ConstructorMap> {
    static CONSTRUCTORS: OnceLock<Mutex<ConstructorMap>> = OnceLock::new();
    CONSTRUCTORS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn responders() -> &'static Mutex<CellGridResponders> {
    static RESPONDERS: OnceLock<Mutex<CellGridResponders>> = OnceLock::new();
    RESPONDERS.get_or_init(|| Mutex::new(CellGridResponders::new()))
}

/// Rust boundary for `vtkCellGridResponderBase`.
pub trait CellGridResponderApi {
    /// VTK: `vtkCellGridResponderBase::EvaluateQuery`.
    fn evaluate_query(
        &mut self,
        query: &mut CellGridQuery,
        cell_type: &mut CellMetadata,
        responders: &mut CellGridResponders,
    ) -> bool;

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str;
}

/// VTK: `vtkCellGridResponders`.
pub struct CellGridResponders {
    object: Object,
    query_responders: ResponderMap,
}

impl CellGridResponders {
    /// VTK: `vtkCellGridResponders::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCellGridResponders"),
            query_responders: HashMap::new(),
        }
    }

    /// VTK: `vtkCellGridResponders::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut out = format!("Responders: ({})\n", self.query_responders.len());
        for (query_type, by_cell) in &self.query_responders {
            out.push_str(&format!(
                "  Query type \"{}\" ({})\n",
                query_type.data(),
                by_cell.len()
            ));
            for (cell_type, responder) in by_cell {
                out.push_str(&format!(
                    "    Cell type \"{}\" -> {}\n",
                    cell_type.data(),
                    responder.get_class_name()
                ));
            }
        }
        out
    }

    /// VTK: `vtkCellGridResponders::RegisterQueryResponder`.
    pub fn register_query_responder(
        &mut self,
        cell_type: StringToken,
        query_type: StringToken,
        responder: Box<dyn CellGridResponderApi + Send>,
    ) {
        self.query_responders
            .entry(query_type)
            .or_default()
            .insert(cell_type, responder);
    }

    /// VTK: `vtkCellGridResponders::Query`.
    pub fn query(&mut self, cell_type: &mut CellMetadata, query: &mut CellGridQuery) -> bool {
        let query_type = StringToken::new_from_str(query.get_class_name());
        let Some(by_cell_type) = self.query_responders.get_mut(&query_type) else {
            return false;
        };

        let mut found_key = None;
        for cell_type_token in cell_type.inheritance_hierarchy() {
            if cell_type_token == StringToken::new_from_str("vtkObject") {
                break;
            }
            if by_cell_type.contains_key(&cell_type_token) {
                found_key = Some(cell_type_token);
                break;
            }
        }

        let Some(found_key) = found_key else {
            return false;
        };
        let Some(mut responder) = by_cell_type.remove(&found_key) else {
            return false;
        };
        let result = responder.evaluate_query(query, cell_type, self);
        self.query_responders
            .entry(query_type)
            .or_default()
            .insert(found_key, responder);
        result
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

impl Default for CellGridResponders {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CellGridResponders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CellGridResponders")
            .field("object", &self.object)
            .field("query_responders", &self.query_responders.len())
            .finish()
    }
}

/// VTK: `vtkCellMetadata`.
#[derive(Debug, Clone)]
pub struct CellMetadata {
    object: Object,
    cell_grid: CellGridMetadataHandle,
}

impl CellMetadata {
    /// VTK: `vtkCellMetadata::vtkCellMetadata`.
    pub fn new() -> Self {
        Self::with_class_name("vtkCellMetadata")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            cell_grid: std::ptr::null_mut(),
        }
    }

    /// VTK: `vtkCellMetadata::RegisterType`.
    pub fn register_type(class_name: &str, constructor: MetadataConstructor) -> bool {
        let token = StringToken::new_from_str(class_name);
        constructors()
            .lock()
            .expect("vtkCellMetadata constructor map")
            .insert(token, constructor)
            .is_none()
    }

    /// VTK: `vtkCellMetadata::NewInstance(vtkStringToken, vtkCellGrid*)`.
    pub fn new_instance(class_name: StringToken, grid: CellGridMetadataHandle) -> Option<Self> {
        let constructor = constructors()
            .lock()
            .expect("vtkCellMetadata constructor map")
            .get(&class_name)
            .copied();
        constructor.map(|ctor| ctor(grid))
    }

    /// VTK: `vtkCellMetadata::CellTypes`.
    pub fn cell_types() -> HashSet<StringToken> {
        constructors()
            .lock()
            .expect("vtkCellMetadata constructor map")
            .keys()
            .copied()
            .collect()
    }

    /// VTK: `vtkCellMetadata::Hash`.
    pub fn hash(&self) -> CellTypeId {
        StringToken::new_from_str(self.get_class_name()).get_id()
    }

    /// VTK: `vtkCellMetadata::SetCellGrid`.
    pub fn set_cell_grid(&mut self, parent: CellGridMetadataHandle) -> bool {
        if self.cell_grid != parent {
            self.cell_grid = parent;
            return true;
        }
        false
    }

    /// VTK: `vtkCellMetadata::GetCellGrid`.
    pub fn get_cell_grid(&self) -> CellGridMetadataHandle {
        self.cell_grid
    }

    /// VTK: `vtkCellMetadata::GetNumberOfCells`.
    pub fn get_number_of_cells(&self) -> VtkIdType {
        0
    }

    /// VTK: `vtkCellMetadata::Query`.
    pub fn query(&mut self, query: &mut CellGridQuery) -> bool {
        Self::get_responders()
            .lock()
            .expect("vtkCellMetadata responders")
            .query(self, query)
    }

    /// VTK: `vtkCellMetadata::ShallowCopy`.
    pub fn shallow_copy(&mut self, _other: &Self) {}

    /// VTK: `vtkCellMetadata::DeepCopy`.
    pub fn deep_copy(&mut self, _other: &Self) {}

    /// VTK: `vtkCellMetadata::GetResponders`.
    pub fn get_responders() -> &'static Mutex<CellGridResponders> {
        responders()
    }

    /// VTK: `vtkCellMetadata::ClearResponders`.
    pub fn clear_responders() {
        *responders().lock().expect("vtkCellMetadata responders") = CellGridResponders::new();
    }

    /// VTK: `vtkCellMetadata::GetCaches`.
    pub fn get_caches(&self) -> MutexGuard<'static, CellGridResponders> {
        let _ = self;
        responders().lock().expect("vtkCellMetadata responders")
    }

    /// VTK: `vtkCellMetadata::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("CellGrid: {:?}\n", self.cell_grid)
    }

    /// VTK: `vtkCellMetadata::InheritanceHierarchy`.
    pub fn inheritance_hierarchy(&self) -> Vec<StringToken> {
        vec![
            StringToken::new_from_str(self.get_class_name()),
            StringToken::new_from_str("vtkCellMetadata"),
            StringToken::new_from_str("vtkObject"),
        ]
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkCellMetadata::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCellMetadata" || Object::is_type_of(name)
    }

    /// VTK: `vtkCellMetadata::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
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

impl Drop for CellMetadata {
    fn drop(&mut self) {
        self.cell_grid = std::ptr::null_mut();
    }
}

impl Default for CellMetadata {
    fn default() -> Self {
        Self::new()
    }
}
