use super::DataObject;
use crate::common::core::{AnyArray, DoubleArray, Object, StringArray, VtkIdType, VtkMTimeType};

/// VTK: `vtkPolyDataMaterial`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyDataMaterial {
    object: Object,
}

impl PolyDataMaterial {
    /// VTK: `vtkPolyDataMaterial::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPolyDataMaterial"),
        }
    }

    /// VTK: `vtkPolyDataMaterial::GetTextureURIName`.
    pub fn get_texture_uri_name() -> &'static str {
        "texture_uri"
    }

    /// VTK: `vtkPolyDataMaterial::GetDiffuseColorName`.
    pub fn get_diffuse_color_name() -> &'static str {
        "diffuse_color"
    }

    /// VTK: `vtkPolyDataMaterial::GetSpecularColorName`.
    pub fn get_specular_color_name() -> &'static str {
        "specular_color"
    }

    /// VTK: `vtkPolyDataMaterial::GetTransparencyName`.
    pub fn get_transparency_name() -> &'static str {
        "transparency"
    }

    /// VTK: `vtkPolyDataMaterial::GetShininessName`.
    pub fn get_shininess_name() -> &'static str {
        "shininess"
    }

    /// VTK: `vtkPolyDataMaterial::SetField(vtkDataObject*, const char*, const char*)`.
    pub fn set_field(obj: &mut DataObject, name: &str, value: &str) {
        let mut array = StringArray::new();
        array.set_number_of_tuples(1);
        array.set_value(0, value);
        array.set_name(name);
        obj.get_field_data_mut().add_array(AnyArray::String(array));
    }

    /// VTK: `vtkPolyDataMaterial::SetField(vtkDataObject*, const char*, const std::vector<std::string>&)`.
    pub fn set_field_values(obj: &mut DataObject, name: &str, values: &[String]) {
        let mut array = StringArray::new();
        array.set_number_of_tuples(values.len() as VtkIdType);
        for (idx, value) in values.iter().enumerate() {
            array.set_value(idx as VtkIdType, value);
        }
        array.set_name(name);
        obj.get_field_data_mut().add_array(AnyArray::String(array));
    }

    /// VTK: `vtkPolyDataMaterial::GetField(vtkDataObject*, const char*)`.
    pub fn get_field(obj: &DataObject, name: &str) -> Vec<String> {
        let Some(AnyArray::String(array)) = obj.get_field_data().get_abstract_array(name) else {
            return Vec::new();
        };

        let mut result = Vec::new();
        for idx in 0..array.get_number_of_tuples() {
            result.push(array.get_value(idx).to_string());
        }
        result
    }

    /// VTK: `vtkPolyDataMaterial::SetField(vtkDataObject*, const char*, double*, vtkIdType)`.
    pub fn set_field_double(
        obj: &mut DataObject,
        name: &str,
        value: &[f64],
        number_of_components: VtkIdType,
    ) {
        let number_of_components = number_of_components.max(1);
        let component_count =
            usize::try_from(number_of_components).expect("component count must fit usize");
        let component_count_i32 =
            i32::try_from(number_of_components).expect("component count must fit i32");

        let mut array = DoubleArray::new();
        array.set_number_of_components(component_count_i32);
        array.set_number_of_tuples(1);
        array.set_typed_tuple(0, &value[..component_count]);
        array.set_name(name);
        obj.get_field_data_mut().add_array(AnyArray::Double(array));
    }

    /// VTK: `vtkPolyDataMaterial::GetField(vtkDataObject*, const char*, const std::vector<double>&)`.
    pub fn get_field_double(obj: &DataObject, name: &str, default_value: &[f64]) -> Vec<f64> {
        let Some(AnyArray::Double(array)) = obj.get_field_data().get_abstract_array(name) else {
            return default_value.to_vec();
        };

        let mut result = vec![0.0; default_value.len()];
        for (dst, src) in result.iter_mut().zip(array.get_typed_tuple(0)) {
            *dst = *src;
        }
        result
    }

    /// VTK: `vtkPolyDataMaterial::PrintSelf`.
    pub fn print_self(&self) -> String {
        String::new()
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

impl Default for PolyDataMaterial {
    fn default() -> Self {
        Self::new()
    }
}
