use std::{cell::RefCell, rc::Rc, sync::OnceLock};

use crate::common::core::{InformationStringKey, Object, VtkMTimeType};

use super::cell_type_utilities::NUMBER_OF_CELL_TYPES;
use super::information_quadrature_scheme_definition_vector_key::InformationQuadratureSchemeDefinitionVectorKey;
use super::{XMLDataElement, XMLDataElementHandle};

const VTK_EMPTY_CELL: i32 = 0;
static DICTIONARY: OnceLock<usize> = OnceLock::new();
static QUADRATURE_OFFSET_ARRAY_NAME: OnceLock<usize> = OnceLock::new();

pub type QuadratureSchemeDefinitionHandle = Rc<RefCell<QuadratureSchemeDefinition>>;

/// VTK: `vtkQuadratureSchemeDefinition`.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadratureSchemeDefinition {
    object: Object,
    cell_type: i32,
    quadrature_key: i32,
    number_of_nodes: i32,
    number_of_quadrature_points: i32,
    dimension: i32,
    shape_function_weights: Vec<f64>,
    quadrature_weights: Vec<f64>,
    shape_function_derivative_weights: Vec<f64>,
}

impl QuadratureSchemeDefinition {
    /// VTK: `vtkQuadratureSchemeDefinition::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkQuadratureSchemeDefinition"),
            cell_type: -1,
            quadrature_key: -1,
            number_of_nodes: 0,
            number_of_quadrature_points: 0,
            dimension: 0,
            shape_function_weights: Vec::new(),
            quadrature_weights: Vec::new(),
            shape_function_derivative_weights: Vec::new(),
        }
    }

    /// VTK: `vtkQuadratureSchemeDefinition::DICTIONARY`.
    pub fn dictionary() -> *mut InformationQuadratureSchemeDefinitionVectorKey {
        *DICTIONARY.get_or_init(|| {
            InformationQuadratureSchemeDefinitionVectorKey::new(
                Some("DICTIONARY"),
                Some("vtkQuadratureSchemeDefinition"),
            ) as usize
        }) as *mut InformationQuadratureSchemeDefinitionVectorKey
    }

    /// VTK: `vtkQuadratureSchemeDefinition::QUADRATURE_OFFSET_ARRAY_NAME`.
    pub fn quadrature_offset_array_name() -> *mut InformationStringKey {
        *QUADRATURE_OFFSET_ARRAY_NAME.get_or_init(|| {
            InformationStringKey::new(
                Some("QUADRATURE_OFFSET_ARRAY_NAME"),
                Some("vtkQuadratureSchemeDefinition"),
            ) as usize
        }) as *mut InformationStringKey
    }

    /// VTK: `vtkQuadratureSchemeDefinition::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) -> i32 {
        self.release_resources();
        self.cell_type = -1;
        self.quadrature_key = -1;
        self.number_of_nodes = 0;
        self.number_of_quadrature_points = 0;

        self.cell_type = other.cell_type;
        self.quadrature_key = other.quadrature_key;
        self.number_of_nodes = other.number_of_nodes;
        self.number_of_quadrature_points = other.number_of_quadrature_points;

        self.secure_resources();
        self.set_shape_function_weights(other.get_shape_function_weights());
        self.set_quadrature_weights(other.get_quadrature_weights());
        1
    }

    /// VTK: `vtkQuadratureSchemeDefinition::SaveState`.
    pub fn save_state(&self, root: &XMLDataElementHandle) -> i32 {
        {
            let root_ref = root.borrow();
            if root_ref.get_name().is_some() || root_ref.get_number_of_nested_elements() > 0 {
                return 0;
            }
        }

        {
            let mut root_ref = root.borrow_mut();
            root_ref.set_name(Some("vtkQuadratureSchemeDefinition"));
        }

        let cell_type = XMLDataElement::new();
        cell_type.borrow_mut().set_name(Some("CellType"));
        cell_type
            .borrow_mut()
            .set_int_attribute(Some("value"), self.cell_type);
        root.borrow_mut().add_nested_element(Some(cell_type));

        let number_of_nodes = XMLDataElement::new();
        number_of_nodes.borrow_mut().set_name(Some("NumberOfNodes"));
        number_of_nodes
            .borrow_mut()
            .set_int_attribute(Some("value"), self.number_of_nodes);
        root.borrow_mut().add_nested_element(Some(number_of_nodes));

        let number_of_quadrature_points = XMLDataElement::new();
        number_of_quadrature_points
            .borrow_mut()
            .set_name(Some("NumberOfQuadraturePoints"));
        number_of_quadrature_points
            .borrow_mut()
            .set_int_attribute(Some("value"), self.number_of_quadrature_points);
        root.borrow_mut()
            .add_nested_element(Some(number_of_quadrature_points));

        let shape_weights = XMLDataElement::new();
        shape_weights
            .borrow_mut()
            .set_name(Some("ShapeFunctionWeights"));
        shape_weights.borrow_mut().set_character_data_width(4);
        root.borrow_mut()
            .add_nested_element(Some(shape_weights.clone()));

        let quadrature_weights = XMLDataElement::new();
        quadrature_weights
            .borrow_mut()
            .set_name(Some("QuadratureWeights"));
        quadrature_weights.borrow_mut().set_character_data_width(4);
        root.borrow_mut()
            .add_nested_element(Some(quadrature_weights.clone()));

        if self.number_of_nodes <= 0 || self.number_of_quadrature_points <= 0 {
            return 0;
        }

        let shape_text = self
            .shape_function_weights
            .iter()
            .take((self.number_of_nodes * self.number_of_quadrature_points) as usize)
            .map(|value| format!("{value:.16e}"))
            .collect::<Vec<_>>()
            .join(" ");
        shape_weights
            .borrow_mut()
            .set_character_data(Some(&shape_text), shape_text.len() as i32);

        let quadrature_text = self
            .quadrature_weights
            .iter()
            .take(self.number_of_quadrature_points as usize)
            .map(|value| format!("{value:.16e}"))
            .collect::<Vec<_>>()
            .join(" ");
        quadrature_weights
            .borrow_mut()
            .set_character_data(Some(&quadrature_text), quadrature_text.len() as i32);
        1
    }

    /// VTK: `vtkQuadratureSchemeDefinition::RestoreState`.
    pub fn restore_state(&mut self, root: &XMLDataElementHandle) -> i32 {
        if root.borrow().get_name() != Some("vtkQuadratureSchemeDefinition") {
            return 0;
        }

        let Some(cell_type) = root
            .borrow()
            .find_nested_element_with_name(Some("CellType"))
        else {
            return 0;
        };
        let value = cell_type
            .borrow()
            .get_attribute(Some("value"))
            .map(ToOwned::to_owned);
        let Some(value) = value else {
            return 0;
        };
        let Ok(cell_type) = value.parse::<i32>() else {
            return 0;
        };

        let Some(number_of_nodes) = root
            .borrow()
            .find_nested_element_with_name(Some("NumberOfNodes"))
        else {
            return 0;
        };
        let value = number_of_nodes
            .borrow()
            .get_attribute(Some("value"))
            .map(ToOwned::to_owned);
        let Some(value) = value else {
            return 0;
        };
        let Ok(number_of_nodes) = value.parse::<i32>() else {
            return 0;
        };

        let Some(number_of_quadrature_points) = root
            .borrow()
            .find_nested_element_with_name(Some("NumberOfQuadraturePoints"))
        else {
            return 0;
        };
        let value = number_of_quadrature_points
            .borrow()
            .get_attribute(Some("value"))
            .map(ToOwned::to_owned);
        let Some(value) = value else {
            return 0;
        };
        let Ok(number_of_quadrature_points) = value.parse::<i32>() else {
            return 0;
        };

        self.cell_type = cell_type;
        self.number_of_nodes = number_of_nodes;
        self.number_of_quadrature_points = number_of_quadrature_points;

        if self.secure_resources() == 0 {
            return 1;
        }

        let Some(shape_weights) = root
            .borrow()
            .find_nested_element_with_name(Some("ShapeFunctionWeights"))
        else {
            return 0;
        };
        let shape_text = shape_weights.borrow().get_character_data().to_owned();
        if !parse_f64_values_exact(&shape_text, &mut self.shape_function_weights) {
            return 0;
        }

        let Some(quadrature_weights) = root
            .borrow()
            .find_nested_element_with_name(Some("QuadratureWeights"))
        else {
            return 0;
        };
        let quadrature_text = quadrature_weights.borrow().get_character_data().to_owned();
        if !parse_f64_values_exact(&quadrature_text, &mut self.quadrature_weights) {
            return 0;
        }
        1
    }

    /// VTK: `vtkQuadratureSchemeDefinition::Initialize`.
    pub fn initialize(
        &mut self,
        cell_type: i32,
        number_of_nodes: i32,
        number_of_quadrature_points: i32,
        shape_function_weights: Option<&[f64]>,
    ) {
        self.release_resources();
        self.cell_type = cell_type;
        self.quadrature_key = -1;
        self.number_of_nodes = number_of_nodes;
        self.number_of_quadrature_points = number_of_quadrature_points;
        self.dimension = 0;
        self.secure_resources();
        self.set_shape_function_weights(shape_function_weights);
    }

    /// VTK: `vtkQuadratureSchemeDefinition::Initialize`.
    pub fn initialize_with_quadrature_weights(
        &mut self,
        cell_type: i32,
        number_of_nodes: i32,
        number_of_quadrature_points: i32,
        shape_function_weights: Option<&[f64]>,
        quadrature_weights: Option<&[f64]>,
    ) {
        self.release_resources();
        self.cell_type = cell_type;
        self.quadrature_key = -1;
        self.number_of_nodes = number_of_nodes;
        self.number_of_quadrature_points = number_of_quadrature_points;
        self.dimension = 0;
        self.secure_resources();
        self.set_shape_function_weights(shape_function_weights);
        self.set_quadrature_weights(quadrature_weights);
    }

    /// VTK: `vtkQuadratureSchemeDefinition::Initialize`.
    pub fn initialize_with_derivative_weights(
        &mut self,
        cell_type: i32,
        number_of_nodes: i32,
        number_of_quadrature_points: i32,
        shape_function_weights: Option<&[f64]>,
        quadrature_weights: Option<&[f64]>,
        dim: i32,
        shape_function_derivative_weights: Option<&[f64]>,
    ) {
        self.release_resources();
        self.cell_type = cell_type;
        self.quadrature_key = -1;
        self.number_of_nodes = number_of_nodes;
        self.number_of_quadrature_points = number_of_quadrature_points;
        self.dimension = dim;
        self.secure_resources();
        self.set_shape_function_weights(shape_function_weights);
        self.set_quadrature_weights(quadrature_weights);
        self.set_shape_function_derivative_weights(shape_function_derivative_weights);
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        self.cell_type
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetQuadratureKey`.
    pub fn get_quadrature_key(&self) -> i32 {
        self.quadrature_key
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetNumberOfNodes`.
    pub fn get_number_of_nodes(&self) -> i32 {
        self.number_of_nodes
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetNumberOfQuadraturePoints`.
    pub fn get_number_of_quadrature_points(&self) -> i32 {
        self.number_of_quadrature_points
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetDimension`.
    pub fn get_dimension(&self) -> i32 {
        self.dimension
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetShapeFunctionWeights`.
    pub fn get_shape_function_weights(&self) -> Option<&[f64]> {
        if self.shape_function_weights.is_empty() {
            None
        } else {
            Some(&self.shape_function_weights)
        }
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetShapeFunctionWeights`.
    pub fn get_shape_function_weights_for_quadrature_point(
        &self,
        quadrature_point_id: i32,
    ) -> Option<&[f64]> {
        if self.number_of_nodes <= 0 || quadrature_point_id < 0 {
            return None;
        }
        let width = self.number_of_nodes as usize;
        let start = quadrature_point_id as usize * width;
        let end = start + width;
        self.shape_function_weights.get(start..end)
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetShapeFunctionDerivativeWeights`.
    pub fn get_shape_function_derivative_weights(
        &self,
        quadrature_point_id: i32,
    ) -> Option<&[f64]> {
        if self.number_of_nodes <= 0 || self.dimension <= 0 || quadrature_point_id < 0 {
            return None;
        }
        let width = (self.number_of_nodes * self.dimension) as usize;
        let start = quadrature_point_id as usize * width;
        let end = start + width;
        self.shape_function_derivative_weights.get(start..end)
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetQuadratureWeights`.
    pub fn get_quadrature_weights(&self) -> Option<&[f64]> {
        if self.quadrature_weights.is_empty() {
            None
        } else {
            Some(&self.quadrature_weights)
        }
    }

    /// VTK: `vtkQuadratureSchemeDefinition::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = format!(
            "{}\nCellType: {}\nQuadratureKey: {}\nNumberOfNodes: {}\nNumberOfQuadraturePoints: {}\nDimension: {}",
            self.object.get_object_description(),
            self.cell_type,
            self.quadrature_key,
            self.number_of_nodes,
            self.number_of_quadrature_points,
            self.dimension
        );
        if self.number_of_nodes > 0 && self.number_of_quadrature_points > 0 {
            let width = self.number_of_nodes as usize;
            for pt_id in 0..self.number_of_quadrature_points as usize {
                let start = pt_id * width;
                let end = start + width;
                if let Some(weights) = self.shape_function_weights.get(start..end) {
                    output.push_str("\n(");
                    for (idx, weight) in weights.iter().enumerate() {
                        if idx > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&weight.to_string());
                    }
                    output.push(')');
                }
            }
        }
        output
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkQuadratureSchemeDefinition::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkQuadratureSchemeDefinition" || Object::is_type_of(name)
    }

    /// VTK: `vtkQuadratureSchemeDefinition::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkQuadratureSchemeDefinition" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkQuadratureSchemeDefinition::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        Object::set_global_warning_display(value);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        Object::global_warning_display_on();
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        Object::global_warning_display_off();
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        Object::get_global_warning_display()
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.object.debug_on();
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.object.debug_off();
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.object.get_debug()
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.object.set_debug(debug);
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {
        Object::break_on_error();
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
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

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.object.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.object.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }

    /// VTK: `vtkQuadratureSchemeDefinition::ReleaseResources`.
    fn release_resources(&mut self) {
        self.shape_function_weights.clear();
        self.quadrature_weights.clear();
        self.shape_function_derivative_weights.clear();
    }

    /// VTK: `vtkQuadratureSchemeDefinition::SecureResources`.
    fn secure_resources(&mut self) -> i32 {
        if self.number_of_quadrature_points <= 0 || self.number_of_nodes <= 0 {
            return 0;
        }

        self.release_resources();
        self.shape_function_weights.resize(
            (self.number_of_quadrature_points * self.number_of_nodes) as usize,
            0.0,
        );
        self.quadrature_weights
            .resize(self.number_of_quadrature_points as usize, 0.0);
        self.shape_function_derivative_weights.resize(
            (self.number_of_quadrature_points * self.number_of_nodes * self.dimension) as usize,
            0.0,
        );
        1
    }

    /// VTK: `vtkQuadratureSchemeDefinition::SetShapeFunctionWeights`.
    fn set_shape_function_weights(&mut self, weights: Option<&[f64]>) {
        if self.number_of_quadrature_points <= 0
            || self.number_of_nodes <= 0
            || self.shape_function_weights.is_empty()
        {
            return;
        }
        let Some(weights) = weights else {
            return;
        };
        let n = (self.number_of_quadrature_points * self.number_of_nodes) as usize;
        for (dest, src) in self
            .shape_function_weights
            .iter_mut()
            .take(n)
            .zip(weights.iter())
        {
            *dest = *src;
        }
    }

    /// VTK: `vtkQuadratureSchemeDefinition::SetQuadratureWeights`.
    fn set_quadrature_weights(&mut self, weights: Option<&[f64]>) {
        if self.number_of_quadrature_points <= 0
            || self.number_of_nodes <= 0
            || self.quadrature_weights.is_empty()
        {
            return;
        }
        let Some(weights) = weights else {
            return;
        };
        for (dest, src) in self.quadrature_weights.iter_mut().zip(weights.iter()) {
            *dest = *src;
        }
    }

    /// VTK: `vtkQuadratureSchemeDefinition::SetShapeFunctionDerivativeWeights`.
    fn set_shape_function_derivative_weights(&mut self, weights: Option<&[f64]>) {
        if self.number_of_quadrature_points <= 0
            || self.number_of_nodes <= 0
            || self.shape_function_derivative_weights.is_empty()
        {
            return;
        }
        let Some(weights) = weights else {
            return;
        };
        for (dest, src) in self
            .shape_function_derivative_weights
            .iter_mut()
            .zip(weights.iter())
        {
            *dest = *src;
        }
    }
}

impl Default for QuadratureSchemeDefinition {
    fn default() -> Self {
        Self::new()
    }
}

/// VTK origin: `VTK/Common/DataModel/vtkCellType.h`.
pub const VTK_NUMBER_OF_CELL_TYPES: i32 = NUMBER_OF_CELL_TYPES;
pub const VTK_EMPTY_CELL_ID: i32 = VTK_EMPTY_CELL;

fn parse_f64_values_exact(source: &str, dest: &mut [f64]) -> bool {
    let mut values = source.split_whitespace();
    for value in dest {
        let Some(token) = values.next() else {
            return false;
        };
        let Ok(parsed) = token.parse::<f64>() else {
            return false;
        };
        *value = parsed;
    }
    true
}
