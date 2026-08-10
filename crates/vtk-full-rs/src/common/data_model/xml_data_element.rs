use std::{
    cell::RefCell,
    fmt::Display,
    rc::{Rc, Weak},
    str::FromStr,
};

use crate::common::core::{Object, ObjectBaseApi, VtkDataType, VtkMTimeType};

pub type XMLDataElementHandle = Rc<RefCell<XMLDataElement>>;

pub const VTK_ENCODING_NONE: i32 = 0;
pub const VTK_ENCODING_UTF_8: i32 = 3;
pub const VTK_ENCODING_UNKNOWN: i32 = 20;

/// VTK: `vtkXMLDataElement`.
#[derive(Debug)]
pub struct XMLDataElement {
    object: Object,
    self_handle: Weak<RefCell<XMLDataElement>>,
    name: Option<String>,
    id: Option<String>,
    character_data_width: i32,
    character_data: String,
    ignore_character_data: bool,
    #[allow(dead_code)]
    inline_data_position: i64,
    xml_byte_index: i64,
    attributes: Vec<(String, String)>,
    attribute_encoding: i32,
    nested_elements: Vec<XMLDataElementHandle>,
    parent: Weak<RefCell<XMLDataElement>>,
}

impl XMLDataElement {
    /// VTK: `vtkXMLDataElement::New`.
    pub fn new() -> XMLDataElementHandle {
        let element = Rc::new(RefCell::new(Self {
            object: Object::with_class_name("vtkXMLDataElement"),
            self_handle: Weak::new(),
            name: None,
            id: None,
            character_data_width: -1,
            character_data: String::new(),
            ignore_character_data: false,
            inline_data_position: 0,
            xml_byte_index: 0,
            attributes: Vec::with_capacity(5),
            attribute_encoding: VTK_ENCODING_UTF_8,
            nested_elements: Vec::with_capacity(10),
            parent: Weak::new(),
        }));
        element.borrow_mut().self_handle = Rc::downgrade(&element);
        element
    }

    /// VTK: `vtkXMLDataElement::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nXMLByteIndex: {}\nName: {}\nId: {}\nNumberOfAttributes: {}\nAttributeEncoding: {}\nCharacterData: {}\nCharacterDataWidth: {}",
            self.object.get_object_description(),
            self.xml_byte_index,
            self.name.as_deref().unwrap_or("(none)"),
            self.id.as_deref().unwrap_or("(none)"),
            self.attributes.len(),
            self.attribute_encoding,
            self.character_data,
            self.character_data_width
        )
    }

    /// VTK: `vtkXMLDataElement::SetName`.
    pub fn set_name(&mut self, name: Option<&str>) {
        if self.name.as_deref() == name {
            return;
        }
        self.ignore_character_data = name.is_some_and(|value| value.contains("DataArray"));
        self.name = name.map(ToOwned::to_owned);
        self.modified();
    }

    /// VTK: `vtkXMLDataElement::GetName`.
    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// VTK: `vtkXMLDataElement::SetId`.
    pub fn set_id(&mut self, id: Option<&str>) {
        if self.id.as_deref() == id {
            return;
        }
        self.id = id.map(ToOwned::to_owned);
        self.modified();
    }

    /// VTK: `vtkXMLDataElement::GetId`.
    pub fn get_id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// VTK: `vtkXMLDataElement::GetAttribute`.
    pub fn get_attribute(&self, name: Option<&str>) -> Option<&str> {
        let name = name?;
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// VTK: `vtkXMLDataElement::SetAttribute`.
    pub fn set_attribute(&mut self, name: Option<&str>, value: Option<&str>) {
        let (Some(name), Some(value)) = (name, value) else {
            return;
        };
        if name.is_empty() || value.is_empty() {
            return;
        }
        if let Some((_, existing_value)) = self.attributes.iter_mut().find(|(key, _)| key == name) {
            *existing_value = value.to_owned();
            return;
        }
        self.attributes.push((name.to_owned(), value.to_owned()));
    }

    /// VTK: `vtkXMLDataElement::SetCharacterData`.
    pub fn set_character_data(&mut self, data: Option<&str>, length: i32) {
        let length = length.max(0) as usize;
        self.character_data.clear();
        if let Some(data) = data {
            let end = data
                .char_indices()
                .map(|(idx, _)| idx)
                .chain(std::iter::once(data.len()))
                .nth(length)
                .unwrap_or(data.len());
            self.character_data.push_str(&data[..end]);
        }
        self.modified();
    }

    /// VTK: `vtkXMLDataElement::AddCharacterData`.
    pub fn add_character_data(&mut self, data: Option<&str>, length: usize) {
        if self.ignore_character_data {
            return;
        }
        let Some(data) = data else {
            return;
        };
        let end = data
            .char_indices()
            .map(|(idx, _)| idx)
            .chain(std::iter::once(data.len()))
            .nth(length)
            .unwrap_or(data.len());
        self.character_data.push_str(&data[..end]);
    }

    /// VTK: `vtkXMLDataElement::GetCharacterData`.
    pub fn get_character_data(&self) -> &str {
        &self.character_data
    }

    /// VTK: `vtkXMLDataElement::GetScalarAttribute`.
    pub fn get_scalar_attribute_i32(&self, name: Option<&str>, value: &mut i32) -> i32 {
        self.get_vector_attribute(name, 1, std::slice::from_mut(value))
    }

    /// VTK: `vtkXMLDataElement::GetScalarAttribute`.
    pub fn get_scalar_attribute_f32(&self, name: Option<&str>, value: &mut f32) -> i32 {
        self.get_vector_attribute(name, 1, std::slice::from_mut(value))
    }

    /// VTK: `vtkXMLDataElement::GetScalarAttribute`.
    pub fn get_scalar_attribute_f64(&self, name: Option<&str>, value: &mut f64) -> i32 {
        self.get_vector_attribute(name, 1, std::slice::from_mut(value))
    }

    /// VTK: `vtkXMLDataElement::GetScalarAttribute`.
    pub fn get_scalar_attribute_i64(&self, name: Option<&str>, value: &mut i64) -> i32 {
        self.get_vector_attribute(name, 1, std::slice::from_mut(value))
    }

    /// VTK: `vtkXMLDataElement::GetScalarAttribute`.
    pub fn get_scalar_attribute_u64(&self, name: Option<&str>, value: &mut u64) -> i32 {
        self.get_vector_attribute(name, 1, std::slice::from_mut(value))
    }

    /// VTK: `vtkXMLDataElement::SetIntAttribute`.
    pub fn set_int_attribute(&mut self, name: Option<&str>, value: i32) {
        self.set_vector_attribute(name, 1, &[value]);
    }

    /// VTK: `vtkXMLDataElement::SetFloatAttribute`.
    pub fn set_float_attribute(&mut self, name: Option<&str>, value: f32) {
        self.set_vector_attribute(name, 1, &[value]);
    }

    /// VTK: `vtkXMLDataElement::SetDoubleAttribute`.
    pub fn set_double_attribute(&mut self, name: Option<&str>, value: f64) {
        self.set_vector_attribute(name, 1, &[value]);
    }

    /// VTK: `vtkXMLDataElement::SetUnsignedLongAttribute`.
    pub fn set_unsigned_long_attribute(&mut self, name: Option<&str>, value: u64) {
        self.set_vector_attribute(name, 1, &[value]);
    }

    /// VTK: `vtkXMLDataElement::GetVectorAttribute`.
    pub fn get_vector_attribute_i32(
        &self,
        name: Option<&str>,
        length: i32,
        value: &mut [i32],
    ) -> i32 {
        self.get_vector_attribute(name, length, value)
    }

    /// VTK: `vtkXMLDataElement::GetVectorAttribute`.
    pub fn get_vector_attribute_f32(
        &self,
        name: Option<&str>,
        length: i32,
        value: &mut [f32],
    ) -> i32 {
        self.get_vector_attribute(name, length, value)
    }

    /// VTK: `vtkXMLDataElement::GetVectorAttribute`.
    pub fn get_vector_attribute_f64(
        &self,
        name: Option<&str>,
        length: i32,
        value: &mut [f64],
    ) -> i32 {
        self.get_vector_attribute(name, length, value)
    }

    /// VTK: `vtkXMLDataElement::GetVectorAttribute`.
    pub fn get_vector_attribute_i64(
        &self,
        name: Option<&str>,
        length: i32,
        value: &mut [i64],
    ) -> i32 {
        self.get_vector_attribute(name, length, value)
    }

    /// VTK: `vtkXMLDataElement::GetVectorAttribute`.
    pub fn get_vector_attribute_u64(
        &self,
        name: Option<&str>,
        length: i32,
        value: &mut [u64],
    ) -> i32 {
        self.get_vector_attribute(name, length, value)
    }

    /// VTK: `vtkXMLDataElement::SetVectorAttribute`.
    pub fn set_vector_attribute_i32(&mut self, name: Option<&str>, length: i32, value: &[i32]) {
        self.set_vector_attribute(name, length, value);
    }

    /// VTK: `vtkXMLDataElement::SetVectorAttribute`.
    pub fn set_vector_attribute_f32(&mut self, name: Option<&str>, length: i32, value: &[f32]) {
        self.set_vector_attribute(name, length, value);
    }

    /// VTK: `vtkXMLDataElement::SetVectorAttribute`.
    pub fn set_vector_attribute_f64(&mut self, name: Option<&str>, length: i32, value: &[f64]) {
        self.set_vector_attribute(name, length, value);
    }

    /// VTK: `vtkXMLDataElement::SetVectorAttribute`.
    pub fn set_vector_attribute_u64(&mut self, name: Option<&str>, length: i32, value: &[u64]) {
        self.set_vector_attribute(name, length, value);
    }

    /// VTK: `vtkXMLDataElement::SetVectorAttribute`.
    pub fn set_vector_attribute_i64(&mut self, name: Option<&str>, length: i32, value: &[i64]) {
        self.set_vector_attribute(name, length, value);
    }

    /// VTK: `vtkXMLDataElement::GetWordTypeAttribute`.
    pub fn get_word_type_attribute(&self, name: Option<&str>, value: &mut i32) -> i32 {
        let Some(attribute) = self.get_attribute(name) else {
            return 0;
        };
        let word_type = match attribute {
            "Float32" => VtkDataType::VTK_FLOAT,
            "Float64" => VtkDataType::VTK_DOUBLE,
            "Int8" => VtkDataType::VTK_SIGNED_CHAR,
            "UInt8" => VtkDataType::VTK_UNSIGNED_CHAR,
            "Int16" => VtkDataType::VTK_SHORT,
            "UInt16" => VtkDataType::VTK_UNSIGNED_SHORT,
            "Int32" => VtkDataType::VTK_INT,
            "UInt32" => VtkDataType::VTK_UNSIGNED_INT,
            "Int64" => VtkDataType::VTK_LONG_LONG,
            "UInt64" => VtkDataType::VTK_UNSIGNED_LONG_LONG,
            "String" => VtkDataType::VTK_STRING,
            "Bit" => VtkDataType::VTK_BIT,
            _ => return 0,
        };
        *value = word_type;
        1
    }

    /// VTK: `vtkXMLDataElement::GetNumberOfAttributes`.
    pub fn get_number_of_attributes(&self) -> i32 {
        self.attributes.len() as i32
    }

    /// VTK: `vtkXMLDataElement::GetAttributeName`.
    pub fn get_attribute_name(&self, idx: i32) -> Option<&str> {
        self.attributes
            .get(usize::try_from(idx).ok()?)
            .map(|(name, _)| name.as_str())
    }

    /// VTK: `vtkXMLDataElement::GetAttributeValue`.
    pub fn get_attribute_value(&self, idx: i32) -> Option<&str> {
        self.attributes
            .get(usize::try_from(idx).ok()?)
            .map(|(_, value)| value.as_str())
    }

    /// VTK: `vtkXMLDataElement::RemoveAttribute`.
    pub fn remove_attribute(&mut self, name: Option<&str>) {
        let Some(name) = name else {
            return;
        };
        if name.is_empty() {
            return;
        }
        if let Some(index) = self.attributes.iter().position(|(key, _)| key == name) {
            self.attributes.remove(index);
        }
    }

    /// VTK: `vtkXMLDataElement::RemoveAllAttributes`.
    pub fn remove_all_attributes(&mut self) {
        self.attributes.clear();
    }

    /// VTK: `vtkXMLDataElement::GetParent`.
    pub fn get_parent(&self) -> Option<XMLDataElementHandle> {
        self.parent.upgrade()
    }

    /// VTK: `vtkXMLDataElement::SetParent`.
    pub fn set_parent(&mut self, parent: Option<XMLDataElementHandle>) {
        self.parent = parent.as_ref().map(Rc::downgrade).unwrap_or_else(Weak::new);
    }

    /// VTK: `vtkXMLDataElement::GetRoot`.
    pub fn get_root(&self) -> Option<XMLDataElementHandle> {
        let mut root = self.self_handle.upgrade()?;
        loop {
            let parent = root.borrow().get_parent();
            let Some(parent) = parent else {
                return Some(root);
            };
            root = parent;
        }
    }

    /// VTK: `vtkXMLDataElement::GetNumberOfNestedElements`.
    pub fn get_number_of_nested_elements(&self) -> i32 {
        self.nested_elements.len() as i32
    }

    /// VTK: `vtkXMLDataElement::GetNestedElement`.
    pub fn get_nested_element(&self, index: i32) -> Option<XMLDataElementHandle> {
        self.nested_elements
            .get(usize::try_from(index).ok()?)
            .cloned()
    }

    /// VTK: `vtkXMLDataElement::AddNestedElement`.
    pub fn add_nested_element(&mut self, element: Option<XMLDataElementHandle>) {
        let Some(element) = element else {
            return;
        };
        if let Some(parent) = self.self_handle.upgrade() {
            element.borrow_mut().set_parent(Some(parent));
        }
        self.nested_elements.push(element);
    }

    /// VTK: `vtkXMLDataElement::RemoveNestedElement`.
    pub fn remove_nested_element(&mut self, element: Option<&XMLDataElementHandle>) {
        let Some(element) = element else {
            return;
        };
        self.nested_elements
            .retain(|current| !Rc::ptr_eq(current, element));
    }

    /// VTK: `vtkXMLDataElement::RemoveAllNestedElements`.
    pub fn remove_all_nested_elements(&mut self) {
        self.nested_elements.clear();
    }

    /// VTK: `vtkXMLDataElement::FindNestedElement`.
    pub fn find_nested_element(&self, id: Option<&str>) -> Option<XMLDataElementHandle> {
        let id = id?;
        self.nested_elements
            .iter()
            .find(|element| element.borrow().get_id() == Some(id))
            .cloned()
    }

    /// VTK: `vtkXMLDataElement::FindNestedElementWithName`.
    pub fn find_nested_element_with_name(
        &self,
        name: Option<&str>,
    ) -> Option<XMLDataElementHandle> {
        let name = name?;
        self.nested_elements
            .iter()
            .find(|element| element.borrow().get_name() == Some(name))
            .cloned()
    }

    /// VTK: `vtkXMLDataElement::FindNestedElementWithNameAndId`.
    pub fn find_nested_element_with_name_and_id(
        &self,
        name: Option<&str>,
        id: Option<&str>,
    ) -> Option<XMLDataElementHandle> {
        let (Some(name), Some(id)) = (name, id) else {
            return None;
        };
        self.nested_elements
            .iter()
            .find(|element| {
                let element = element.borrow();
                element.get_name() == Some(name) && element.get_id() == Some(id)
            })
            .cloned()
    }

    /// VTK: `vtkXMLDataElement::FindNestedElementWithNameAndAttribute`.
    pub fn find_nested_element_with_name_and_attribute(
        &self,
        name: Option<&str>,
        att_name: Option<&str>,
        att_value: Option<&str>,
    ) -> Option<XMLDataElementHandle> {
        let (Some(name), Some(att_name), Some(att_value)) = (name, att_name, att_value) else {
            return None;
        };
        self.nested_elements
            .iter()
            .find(|element| {
                let element = element.borrow();
                element.get_name() == Some(name)
                    && element.get_attribute(Some(att_name)) == Some(att_value)
            })
            .cloned()
    }

    /// VTK: `vtkXMLDataElement::LookupElementWithName`.
    pub fn lookup_element_with_name(&self, name: Option<&str>) -> Option<XMLDataElementHandle> {
        let name = name?;
        for element in &self.nested_elements {
            if element.borrow().get_name() == Some(name) {
                return Some(element.clone());
            }
            if let Some(found) = element.borrow().lookup_element_with_name(Some(name)) {
                return Some(found);
            }
        }
        None
    }

    /// VTK: `vtkXMLDataElement::LookupElement`.
    pub fn lookup_element(&self, id: Option<&str>) -> Option<XMLDataElementHandle> {
        self.lookup_element_up_scope(id)
    }

    /// VTK: `vtkXMLDataElement::GetXMLByteIndex`.
    pub fn get_xml_byte_index(&self) -> i64 {
        self.xml_byte_index
    }

    /// VTK: `vtkXMLDataElement::SetXMLByteIndex`.
    pub fn set_xml_byte_index(&mut self, xml_byte_index: i64) {
        self.xml_byte_index = xml_byte_index;
    }

    /// VTK: `vtkXMLDataElement::IsEqualTo`.
    pub fn is_equal_to(&self, elem: Option<&XMLDataElementHandle>) -> i32 {
        let Some(elem) = elem else {
            return 0;
        };
        let elem = elem.borrow();
        if self.name != elem.name
            || self.character_data != elem.character_data
            || self.attributes.len() != elem.attributes.len()
            || self.nested_elements.len() != elem.nested_elements.len()
        {
            return 0;
        }
        for (name, value) in &self.attributes {
            if elem.get_attribute(Some(name)) != Some(value.as_str()) {
                return 0;
            }
        }
        for (left, right) in self.nested_elements.iter().zip(elem.nested_elements.iter()) {
            if left.borrow().is_equal_to(Some(right)) == 0 {
                return 0;
            }
        }
        1
    }

    /// VTK: `vtkXMLDataElement::DeepCopy`.
    pub fn deep_copy(&mut self, elem: Option<&XMLDataElementHandle>) {
        let Some(elem) = elem else {
            return;
        };
        let elem = elem.borrow();
        self.set_name(elem.get_name());
        self.set_id(elem.get_id());
        self.set_xml_byte_index(elem.get_xml_byte_index());
        self.set_attribute_encoding(elem.get_attribute_encoding());
        self.set_character_data(
            Some(elem.get_character_data()),
            elem.get_character_data().len() as i32,
        );
        self.set_character_data_width(elem.get_character_data_width());
        self.remove_all_attributes();
        for (name, value) in &elem.attributes {
            self.set_attribute(Some(name), Some(value));
        }
        self.remove_all_nested_elements();
        for nested in &elem.nested_elements {
            let nested_copy = XMLDataElement::new();
            nested_copy.borrow_mut().deep_copy(Some(nested));
            self.add_nested_element(Some(nested_copy));
        }
    }

    /// VTK: `vtkXMLDataElement::GetAttributeEncoding`.
    pub fn get_attribute_encoding(&self) -> i32 {
        self.attribute_encoding
    }

    /// VTK: `vtkXMLDataElement::SetAttributeEncoding`.
    pub fn set_attribute_encoding(&mut self, attribute_encoding: i32) {
        self.attribute_encoding = attribute_encoding.clamp(VTK_ENCODING_NONE, VTK_ENCODING_UNKNOWN);
    }

    /// VTK: `vtkXMLDataElement::PrintXML`.
    pub fn print_xml(&self) -> String {
        self.print_xml_with_indent(0)
    }

    /// VTK: `vtkXMLDataElement::GetCharacterDataWidth`.
    pub fn get_character_data_width(&self) -> i32 {
        self.character_data_width
    }

    /// VTK: `vtkXMLDataElement::SetCharacterDataWidth`.
    pub fn set_character_data_width(&mut self, character_data_width: i32) {
        self.character_data_width = character_data_width;
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkXMLDataElement::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkXMLDataElement" || Object::is_type_of(name)
    }

    /// VTK: `vtkXMLDataElement::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkXMLDataElement::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkXMLDataElement" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkXMLDataElement::GetNumberOfGenerationsFromBase`.
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

    /// VTK: `vtkXMLDataElement::LookupElementInScope`.
    fn lookup_element_in_scope(&self, id: Option<&str>) -> Option<XMLDataElementHandle> {
        let id = id?;
        let (name, rest) = split_xml_lookup_id(id);
        let next = self.find_nested_element(Some(name));
        match (next, rest) {
            (Some(next), Some(rest)) => next.borrow().lookup_element_in_scope(Some(rest)),
            (next, None) => next,
            (None, _) => None,
        }
    }

    /// VTK: `vtkXMLDataElement::LookupElementUpScope`.
    fn lookup_element_up_scope(&self, id: Option<&str>) -> Option<XMLDataElementHandle> {
        let id = id?;
        let (name, rest) = split_xml_lookup_id(id);
        let mut scope = self.self_handle.upgrade();
        while let Some(current) = scope {
            if let Some(start) = current.borrow().find_nested_element(Some(name)) {
                return match rest {
                    Some(rest) => start.borrow().lookup_element_in_scope(Some(rest)),
                    None => Some(start),
                };
            }
            scope = current.borrow().get_parent();
        }
        None
    }

    /// VTK: `vtkXMLDataElement::IsSpace`.
    #[allow(dead_code)]
    fn is_space(c: char) -> i32 {
        c.is_whitespace() as i32
    }

    /// VTK: `vtkXMLDataElement::PrintCharacterData`.
    fn print_character_data(&self, indent: usize) -> String {
        if self.character_data.is_empty() {
            return String::new();
        }
        let padding = " ".repeat(indent);
        if self.character_data_width < 1 {
            return format!(
                "{}{}\n",
                padding,
                Self::print_with_escaped_data(&self.character_data)
            );
        }
        let mut output = String::new();
        output.push_str(&padding);
        for (idx, token) in self.character_data.split_whitespace().enumerate() {
            if idx > 0 {
                if idx % self.character_data_width as usize == 0 {
                    output.push('\n');
                    output.push_str(&padding);
                } else {
                    output.push(' ');
                }
            }
            output.push_str(&Self::print_with_escaped_data(token));
        }
        output.push('\n');
        output
    }

    /// VTK: `vtkXMLDataElement::PrintWithEscapedData`.
    fn print_with_escaped_data(data: &str) -> String {
        data.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    fn print_xml_with_indent(&self, indent: usize) -> String {
        let padding = " ".repeat(indent);
        let next_indent = indent + 2;
        let mut output = format!("{}<{}", padding, self.name.as_deref().unwrap_or(""));
        for (name, value) in &self.attributes {
            output.push_str(&format!(
                " {}=\"{}\"",
                name,
                Self::print_with_escaped_data(value)
            ));
        }
        if !self.nested_elements.is_empty() || !self.character_data.is_empty() {
            output.push_str(">\n");
            for element in &self.nested_elements {
                output.push_str(&element.borrow().print_xml_with_indent(next_indent));
            }
            output.push_str(&self.print_character_data(next_indent));
            output.push_str(&format!(
                "{}</{}>\n",
                padding,
                self.name.as_deref().unwrap_or("")
            ));
        } else {
            output.push_str("/>\n");
        }
        output
    }

    fn get_vector_attribute<T>(&self, name: Option<&str>, length: i32, data: &mut [T]) -> i32
    where
        T: FromStr,
    {
        if length <= 0 {
            return 0;
        }
        let Some(attribute) = self.get_attribute(name) else {
            return 0;
        };
        let mut read = 0;
        for (idx, token) in attribute
            .split_whitespace()
            .take(length as usize)
            .enumerate()
        {
            let Ok(value) = token.parse::<T>() else {
                return read;
            };
            if let Some(dest) = data.get_mut(idx) {
                *dest = value;
            }
            read += 1;
        }
        read
    }

    fn set_vector_attribute<T>(&mut self, name: Option<&str>, length: i32, data: &[T])
    where
        T: Display,
    {
        let Some(name) = name else {
            return;
        };
        if length <= 0 || data.is_empty() {
            return;
        }
        let value = data
            .iter()
            .take(length as usize)
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        self.set_attribute(Some(name), Some(&value));
    }
}

impl ObjectBaseApi for XMLDataElement {
    fn get_class_name(&self) -> &str {
        self.get_class_name()
    }

    fn is_a(&self, name: &str) -> bool {
        self.is_a(name)
    }

    fn get_object_description(&self) -> String {
        self.get_object_description()
    }

    fn print_self(&self) -> String {
        self.print_self()
    }
}

fn split_xml_lookup_id(id: &str) -> (&str, Option<&str>) {
    id.split_once('.')
        .map(|(left, right)| (left, Some(right)))
        .unwrap_or((id, None))
}
