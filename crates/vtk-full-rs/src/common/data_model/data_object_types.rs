use crate::common::core::Object;

pub const VTK_POLY_DATA: i32 = 0;
pub const VTK_STRUCTURED_POINTS: i32 = 1;
pub const VTK_STRUCTURED_GRID: i32 = 2;
pub const VTK_RECTILINEAR_GRID: i32 = 3;
pub const VTK_UNSTRUCTURED_GRID: i32 = 4;
pub const VTK_PIECEWISE_FUNCTION: i32 = 5;
pub const VTK_IMAGE_DATA: i32 = 6;
pub const VTK_DATA_OBJECT: i32 = 7;
pub const VTK_DATA_SET: i32 = 8;
pub const VTK_POINT_SET: i32 = 9;
pub const VTK_UNIFORM_GRID: i32 = 10;
pub const VTK_COMPOSITE_DATA_SET: i32 = 11;
pub const VTK_MULTIBLOCK_DATA_SET: i32 = 13;
pub const VTK_GENERIC_DATA_SET: i32 = 16;
pub const VTK_TABLE: i32 = 19;
pub const VTK_GRAPH: i32 = 20;
pub const VTK_TREE: i32 = 21;
pub const VTK_SELECTION: i32 = 22;
pub const VTK_DIRECTED_GRAPH: i32 = 23;
pub const VTK_UNDIRECTED_GRAPH: i32 = 24;
pub const VTK_MULTIPIECE_DATA_SET: i32 = 25;
pub const VTK_DIRECTED_ACYCLIC_GRAPH: i32 = 26;
pub const VTK_ARRAY_DATA: i32 = 27;
pub const VTK_REEB_GRAPH: i32 = 28;
pub const VTK_UNIFORM_GRID_AMR: i32 = 29;
pub const VTK_NON_OVERLAPPING_AMR: i32 = 30;
pub const VTK_OVERLAPPING_AMR: i32 = 31;
pub const VTK_HYPER_TREE_GRID: i32 = 32;
pub const VTK_MOLECULE: i32 = 33;
pub const VTK_PATH: i32 = 35;
pub const VTK_UNSTRUCTURED_GRID_BASE: i32 = 36;
pub const VTK_PARTITIONED_DATA_SET: i32 = 37;
pub const VTK_PARTITIONED_DATA_SET_COLLECTION: i32 = 38;
pub const VTK_UNIFORM_HYPER_TREE_GRID: i32 = 39;
pub const VTK_EXPLICIT_STRUCTURED_GRID: i32 = 40;
pub const VTK_DATA_OBJECT_TREE: i32 = 41;
pub const VTK_ABSTRACT_ELECTRONIC_DATA: i32 = 42;
pub const VTK_OPEN_QUBE_ELECTRONIC_DATA: i32 = 43;
pub const VTK_ANNOTATION: i32 = 44;
pub const VTK_ANNOTATION_LAYERS: i32 = 45;
pub const VTK_BSP_CUTS: i32 = 46;
pub const VTK_GEO_JSON_FEATURE: i32 = 47;
pub const VTK_IMAGE_STENCIL_DATA: i32 = 48;
pub const VTK_CELL_GRID: i32 = 49;
pub const VTK_AMR_DATA_OBJECT: i32 = 50;
pub const VTK_CARTESIAN_GRID: i32 = 51;
pub const VTK_STATISTICAL_MODEL: i32 = 52;

const DATA_OBJECT_TYPE_STRINGS: [&str; 53] = [
    "vtkPolyData",
    "vtkStructuredPoints",
    "vtkStructuredGrid",
    "vtkRectilinearGrid",
    "vtkUnstructuredGrid",
    "vtkPiecewiseFunction",
    "vtkImageData",
    "vtkDataObject",
    "vtkDataSet",
    "vtkPointSet",
    "vtkUniformGrid",
    "vtkCompositeDataSet",
    "vtkMultiGroupDataSet",
    "vtkMultiBlockDataSet",
    "vtkHierarchicalDataSet",
    "vtkHierarchicalBoxDataSet",
    "vtkGenericDataSet",
    "vtkHyperOctree",
    "vtkTemporalDataSet",
    "vtkTable",
    "vtkGraph",
    "vtkTree",
    "vtkSelection",
    "vtkDirectedGraph",
    "vtkUndirectedGraph",
    "vtkMultiPieceDataSet",
    "vtkDirectedAcyclicGraph",
    "vtkArrayData",
    "vtkReebGraph",
    "vtkUniformGridAMR",
    "vtkNonOverlappingAMR",
    "vtkOverlappingAMR",
    "vtkHyperTreeGrid",
    "vtkMolecule",
    "vtkPistonDataObject",
    "vtkPath",
    "vtkUnstructuredGridBase",
    "vtkPartitionedDataSet",
    "vtkPartitionedDataSetCollection",
    "vtkUniformHyperTreeGrid",
    "vtkExplicitStructuredGrid",
    "vtkDataObjectTree",
    "vtkAbstractElectronicData",
    "vtkOpenQubeElectronicData",
    "vtkAnnotation",
    "vtkAnnotationLayers",
    "vtkBSPCuts",
    "vtkGeoJSONFeature",
    "vtkImageStencilData",
    "vtkCellGrid",
    "vtkAMRDataObject",
    "vtkCartesianGrid",
    "vtkStatisticalModel",
];

#[derive(Debug, Clone, PartialEq)]
pub struct DataObjectTypes {
    object: Object,
}

impl DataObjectTypes {
    /// VTK: `vtkDataObjectTypes::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDataObjectTypes"),
        }
    }

    /// VTK: `vtkDataObjectTypes::GetClassNameFromTypeId`.
    pub fn get_class_name_from_type_id(type_id: i32) -> &'static str {
        if is_type_id_valid(type_id) {
            DATA_OBJECT_TYPE_STRINGS[type_id as usize]
        } else {
            "UnknownClass"
        }
    }

    /// VTK: `vtkDataObjectTypes::GetTypeIdFromClassName`.
    pub fn get_type_id_from_class_name(classname: Option<&str>) -> i32 {
        let Some(classname) = classname else {
            return -1;
        };
        DATA_OBJECT_TYPE_STRINGS
            .iter()
            .position(|name| *name == classname)
            .map_or(-1, |idx| idx as i32)
    }

    /// VTK: `vtkDataObjectTypes::TypeIdIsA`.
    pub fn type_id_is_a(type_id: i32, target_type_id: i32) -> bool {
        if !is_type_id_valid(type_id) || !is_type_id_valid(target_type_id) {
            return false;
        }

        type_id == target_type_id
            || Self::get_common_base_type_id(type_id, target_type_id) == target_type_id
    }

    /// VTK: `vtkDataObjectTypes::GetCommonBaseTypeId`.
    pub fn get_common_base_type_id(type_a: i32, type_b: i32) -> i32 {
        match (is_type_id_valid(type_a), is_type_id_valid(type_b)) {
            (false, false) => return -1,
            (false, true) => return type_b,
            (true, false) => return type_a,
            (true, true) => {}
        }

        let branch_a = compute_branch(type_a);
        let branch_b = compute_branch(type_b);
        branch_a
            .iter()
            .zip(branch_b.iter())
            .take_while(|(a, b)| a == b)
            .map(|(a, _)| *a)
            .last()
            .unwrap_or(VTK_DATA_OBJECT)
    }

    /// VTK: `vtkDataObjectTypes::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.get_class_name().to_string()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkDataObjectTypes::Validate`.
    #[allow(dead_code)]
    pub(crate) fn validate() -> i32 {
        let hierarchy_matches = Self::type_id_is_a(VTK_DATA_SET, VTK_DATA_OBJECT)
            && !Self::type_id_is_a(VTK_DATA_SET, VTK_TABLE)
            && Self::type_id_is_a(VTK_PARTITIONED_DATA_SET_COLLECTION, VTK_COMPOSITE_DATA_SET)
            && Self::type_id_is_a(VTK_MULTIBLOCK_DATA_SET, VTK_DATA_OBJECT_TREE)
            && Self::type_id_is_a(VTK_UNIFORM_GRID_AMR, VTK_AMR_DATA_OBJECT)
            && Self::type_id_is_a(VTK_OVERLAPPING_AMR, VTK_UNIFORM_GRID_AMR)
            && Self::type_id_is_a(VTK_UNSTRUCTURED_GRID, VTK_POINT_SET)
            && Self::type_id_is_a(VTK_UNSTRUCTURED_GRID, VTK_DATA_SET)
            && Self::type_id_is_a(VTK_CELL_GRID, VTK_DATA_OBJECT)
            && Self::type_id_is_a(VTK_STATISTICAL_MODEL, VTK_DATA_OBJECT);

        i32::from(!hierarchy_matches)
    }
}

impl Default for DataObjectTypes {
    fn default() -> Self {
        Self::new()
    }
}

fn is_type_id_valid(type_id: i32) -> bool {
    (VTK_POLY_DATA..=VTK_STATISTICAL_MODEL).contains(&type_id)
}

fn compute_branch(mut type_id: i32) -> Vec<i32> {
    let mut branch = Vec::new();
    loop {
        branch.push(type_id);
        let next = immediate_base(type_id).unwrap_or(VTK_DATA_OBJECT);
        if next == VTK_DATA_OBJECT {
            break;
        }
        type_id = next;
    }
    branch.push(VTK_DATA_OBJECT);
    branch.reverse();
    branch
}

fn immediate_base(type_id: i32) -> Option<i32> {
    match type_id {
        VTK_UNIFORM_HYPER_TREE_GRID => Some(VTK_HYPER_TREE_GRID),
        VTK_UNDIRECTED_GRAPH => Some(VTK_GRAPH),
        VTK_DIRECTED_GRAPH => Some(VTK_GRAPH),
        VTK_MOLECULE => Some(VTK_UNDIRECTED_GRAPH),
        VTK_DIRECTED_ACYCLIC_GRAPH => Some(VTK_DIRECTED_GRAPH),
        VTK_REEB_GRAPH => Some(VTK_DIRECTED_GRAPH),
        VTK_TREE => Some(VTK_DIRECTED_ACYCLIC_GRAPH),
        VTK_RECTILINEAR_GRID => Some(VTK_CARTESIAN_GRID),
        VTK_POINT_SET => Some(VTK_DATA_SET),
        VTK_CARTESIAN_GRID => Some(VTK_DATA_SET),
        VTK_IMAGE_DATA => Some(VTK_CARTESIAN_GRID),
        VTK_UNSTRUCTURED_GRID_BASE => Some(VTK_POINT_SET),
        VTK_STRUCTURED_GRID => Some(VTK_POINT_SET),
        VTK_POLY_DATA => Some(VTK_POINT_SET),
        VTK_PATH => Some(VTK_POINT_SET),
        VTK_EXPLICIT_STRUCTURED_GRID => Some(VTK_POINT_SET),
        VTK_UNSTRUCTURED_GRID => Some(VTK_UNSTRUCTURED_GRID_BASE),
        VTK_UNIFORM_GRID => Some(VTK_IMAGE_DATA),
        VTK_STRUCTURED_POINTS => Some(VTK_IMAGE_DATA),
        VTK_UNIFORM_GRID_AMR => Some(VTK_AMR_DATA_OBJECT),
        VTK_OVERLAPPING_AMR => Some(VTK_UNIFORM_GRID_AMR),
        VTK_NON_OVERLAPPING_AMR => Some(VTK_UNIFORM_GRID_AMR),
        VTK_DATA_OBJECT_TREE => Some(VTK_COMPOSITE_DATA_SET),
        VTK_PARTITIONED_DATA_SET_COLLECTION => Some(VTK_DATA_OBJECT_TREE),
        VTK_PARTITIONED_DATA_SET => Some(VTK_DATA_OBJECT_TREE),
        VTK_MULTIPIECE_DATA_SET => Some(VTK_PARTITIONED_DATA_SET),
        VTK_MULTIBLOCK_DATA_SET => Some(VTK_DATA_OBJECT_TREE),
        VTK_OPEN_QUBE_ELECTRONIC_DATA => Some(VTK_ABSTRACT_ELECTRONIC_DATA),
        VTK_STATISTICAL_MODEL => Some(VTK_DATA_OBJECT),
        _ => None,
    }
}
