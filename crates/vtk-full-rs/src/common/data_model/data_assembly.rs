const DATASET_NODE_NAME: &str = "dataset";
const ROOT_NODE_ID: i32 = 0;
const ROOT_NODE_NAME: &str = "assembly";

fn usize_from_id(id: i32) -> Option<usize> {
    usize::try_from(id).ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataAssemblyNode {
    name: String,
    parent: Option<i32>,
    children: Vec<i32>,
    dataset_indices: Vec<u32>,
    attributes: Vec<(String, String)>,
}

impl DataAssemblyNode {
    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    fn set_attribute(&mut self, name: &str, value: &str) {
        if let Some((_, old_value)) = self.attributes.iter_mut().find(|(key, _)| key == name) {
            old_value.clear();
            old_value.push_str(value);
        } else {
            self.attributes.push((name.to_string(), value.to_string()));
        }
    }
}

/// VTK: `vtkDataAssembly::TraversalOrder`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalOrder {
    DepthFirst,
    BreadthFirst,
}

/// Current traversal state for `DataAssemblyVisitor` callbacks.
///
/// VTK origin: `vtkDataAssemblyVisitor`.
pub struct DataAssemblyVisitContext<'a> {
    assembly: &'a DataAssembly,
    current_node: i32,
    traversal_order: TraversalOrder,
}

impl<'a> DataAssemblyVisitContext<'a> {
    /// VTK: `vtkDataAssemblyVisitor::GetAssembly`.
    pub fn get_assembly(&self) -> &'a DataAssembly {
        self.assembly
    }

    /// VTK: `vtkDataAssemblyVisitor::GetTraversalOrder`.
    pub fn get_traversal_order(&self) -> TraversalOrder {
        self.traversal_order
    }

    /// VTK: `vtkDataAssemblyVisitor::GetCurrentNodeName`.
    pub fn get_current_node_name(&self) -> Option<&'a str> {
        self.assembly.get_node_name(self.current_node)
    }

    /// VTK: `vtkDataAssemblyVisitor::GetCurrentNodePath`.
    pub fn get_current_node_path(&self) -> Vec<String> {
        self.assembly
            .get_node_path_components(self.current_node)
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    /// VTK: `vtkDataAssemblyVisitor::GetCurrentDataSetIndices`.
    pub fn get_current_data_set_indices(&self) -> Vec<u32> {
        self.assembly
            .node(self.current_node)
            .map(|node| node.dataset_indices.clone())
            .unwrap_or_default()
    }
}

/// Visitor callbacks for `DataAssembly::visit`.
///
/// VTK origin: `vtkDataAssemblyVisitor`.
pub trait DataAssemblyVisitor {
    /// VTK: `vtkDataAssemblyVisitor::Visit`.
    fn visit(&mut self, node_id: i32, context: &DataAssemblyVisitContext<'_>);

    /// VTK: `vtkDataAssemblyVisitor::GetTraverseSubtree`.
    fn get_traverse_subtree(
        &mut self,
        _node_id: i32,
        _context: &DataAssemblyVisitContext<'_>,
    ) -> bool {
        true
    }

    /// VTK: `vtkDataAssemblyVisitor::BeginSubTree`.
    fn begin_sub_tree(&mut self, _node_id: i32, _context: &DataAssemblyVisitContext<'_>) {}

    /// VTK: `vtkDataAssemblyVisitor::EndSubTree`.
    fn end_sub_tree(&mut self, _node_id: i32, _context: &DataAssemblyVisitContext<'_>) {}
}

/// Hierarchical organization of data items.
///
/// VTK origin: `VTK/Common/DataModel/vtkDataAssembly.{h,cxx}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataAssembly {
    nodes: Vec<Option<DataAssemblyNode>>,
    max_unique_id: i32,
    modified_time: u64,
}

impl DataAssembly {
    /// VTK: `vtkDataAssembly::New`.
    pub fn new() -> Self {
        let mut assembly = Self {
            nodes: Vec::new(),
            max_unique_id: ROOT_NODE_ID,
            modified_time: 0,
        };
        assembly.initialize();
        assembly
    }

    /// VTK: `vtkDataAssembly::Initialize`.
    pub fn initialize(&mut self) {
        self.nodes.clear();
        self.nodes.push(Some(DataAssemblyNode {
            name: ROOT_NODE_NAME.to_string(),
            parent: None,
            children: Vec::new(),
            dataset_indices: Vec::new(),
            attributes: Vec::new(),
        }));
        self.max_unique_id = ROOT_NODE_ID;
        self.modified();
    }

    /// VTK: `vtkDataAssembly::SerializeToXML`.
    pub fn serialize_to_xml(&self, indent: &str) -> String {
        let mut xml = String::new();
        self.serialize_node(Self::get_root_node(), indent, 0, &mut xml);
        xml
    }

    /// VTK: `vtkDataAssembly::GetRootNode`.
    pub fn get_root_node() -> i32 {
        ROOT_NODE_ID
    }

    /// VTK: `vtkDataAssembly::SetRootNodeName`.
    pub fn set_root_node_name(&mut self, name: Option<&str>) {
        self.set_node_name(Self::get_root_node(), name);
    }

    /// VTK: `vtkDataAssembly::GetRootNodeName`.
    pub fn get_root_node_name(&self) -> Option<&str> {
        self.get_node_name(Self::get_root_node())
    }

    /// VTK: `vtkDataAssembly::AddNode`.
    pub fn add_node(&mut self, name: Option<&str>, parent: i32) -> i32 {
        let Some(name) = name.filter(|name| Self::is_node_name_valid(Some(name))) else {
            return -1;
        };
        if self.node(parent).is_none() {
            return -1;
        }
        let child = self.append_node(parent, name);
        self.modified();
        child
    }

    /// VTK: `vtkDataAssembly::AddNodes`.
    pub fn add_nodes(&mut self, names: &[String], parent: i32) -> Vec<i32> {
        if self.node(parent).is_none() {
            return Vec::new();
        }
        if names
            .iter()
            .any(|name| !Self::is_node_name_valid(Some(name.as_str())))
        {
            return Vec::new();
        }

        let mut ids = Vec::with_capacity(names.len());
        for name in names {
            ids.push(self.append_node(parent, name));
        }
        if !ids.is_empty() {
            self.modified();
        }
        ids
    }

    /// VTK: `vtkDataAssembly::RemoveNode`.
    pub fn remove_node(&mut self, id: i32) -> bool {
        if id == Self::get_root_node() || self.node(id).is_none() {
            return false;
        }

        let parent = self.node(id).and_then(|node| node.parent);
        if let Some(parent) = parent.and_then(|parent| self.node_mut(parent)) {
            parent.children.retain(|&child| child != id);
        }
        self.remove_subtree(id);
        self.modified();
        true
    }

    /// VTK: `vtkDataAssembly::SetNodeName`.
    pub fn set_node_name(&mut self, id: i32, name: Option<&str>) {
        let Some(name) = name.filter(|name| Self::is_node_name_valid(Some(name))) else {
            return;
        };
        let Some(node) = self.node_mut(id) else {
            return;
        };
        node.name.clear();
        node.name.push_str(name);
        self.modified();
    }

    /// VTK: `vtkDataAssembly::GetNodeName`.
    pub fn get_node_name(&self, id: i32) -> Option<&str> {
        self.node(id).map(|node| node.name.as_str())
    }

    /// VTK: `vtkDataAssembly::GetNodePath`.
    pub fn get_node_path(&self, id: i32) -> String {
        if self.node(id).is_none() {
            return String::new();
        }

        let mut names = Vec::new();
        let mut current = Some(id);
        while let Some(node_id) = current {
            let Some(node) = self.node(node_id) else {
                return String::new();
            };
            names.push(node.name.as_str());
            current = node.parent;
        }
        names.reverse();

        let mut path = String::new();
        for name in names {
            path.push('/');
            path.push_str(name);
        }
        path
    }

    /// VTK: `vtkDataAssembly::GetFirstNodeByPath`.
    pub fn get_first_node_by_path(&self, path: Option<&str>) -> i32 {
        let Some(path) = path else {
            return -1;
        };

        self.first_node_by_pugi_path(path).unwrap_or(-1)
    }

    /// VTK: `vtkDataAssembly::FindOrCreateNodeAtPath`.
    pub fn find_or_create_node_at_path(&mut self, path: Option<&str>, parent: i32) -> i32 {
        let Some(path) = path.filter(|path| !path.is_empty()) else {
            return parent;
        };
        if self.node(parent).is_none() {
            return -1;
        }

        let path_names = split_path_preserve_empty(path);
        let mut node = parent;
        let mut index = 0;
        while index < path_names.len() {
            let child_name = path_names[index];
            if let Some(child) = self.child_with_name(node, child_name) {
                node = child;
                index += 1;
            } else {
                while index < path_names.len() {
                    node = self.add_node(Some(path_names[index]), node);
                    if node == -1 {
                        return -1;
                    }
                    index += 1;
                }
                return node;
            }
        }
        node
    }

    /// VTK: `vtkDataAssembly::DeepCopy`.
    pub fn deep_copy(&mut self, other: Option<&Self>) {
        if let Some(other) = other {
            self.nodes = other.nodes.clone();
            self.max_unique_id = other.max_unique_id;
            self.modified();
        } else {
            self.initialize();
        }
    }

    /// VTK: `vtkDataAssembly::AddSubtree`.
    pub fn add_subtree(&mut self, parent: i32, other: Option<&Self>, other_parent: i32) -> i32 {
        let Some(other) = other else {
            return -1;
        };
        if self.node(parent).is_none() || other.node(other_parent).is_none() {
            return -1;
        }

        let node_offset = self.max_unique_id.saturating_add(1);
        let dataset_offset = self
            .max_data_set_index()
            .map_or(0, |id| id.saturating_add(1));
        let new_root = other_parent.saturating_add(node_offset);
        let copied = other.copy_subtree_nodes(other_parent, None, node_offset, dataset_offset);

        if let Some(parent_node) = self.node_mut(parent) {
            parent_node.children.push(new_root);
        }
        for (id, mut node) in copied {
            if id == new_root && other_parent == Self::get_root_node() {
                node.attributes
                    .retain(|(name, _)| name != "type" && name != "version");
            }
            self.set_copied_node(id, node);
        }
        self.refresh_max_unique_id();
        self.modified();
        1
    }

    /// VTK: `vtkDataAssembly::SubsetCopy`.
    pub fn subset_copy(&mut self, other: Option<&Self>, selected_branches: &[i32]) {
        self.initialize();
        let Some(other) = other else {
            return;
        };

        if selected_branches.is_empty() {
            self.copy_root_shell_from(other, false);
            return;
        }

        let mut complete_subtree = std::collections::BTreeSet::new();
        let mut partial_subtree = std::collections::BTreeSet::new();
        for &id in selected_branches {
            if other.node(id).is_some() {
                complete_subtree.insert(id);
                let mut node = other.get_parent(id);
                while node != -1 {
                    if !partial_subtree.insert(node) {
                        break;
                    }
                    node = other.get_parent(node);
                }
            }
        }

        if complete_subtree.contains(&Self::get_root_node()) {
            self.deep_copy(Some(other));
            return;
        }

        if !partial_subtree.contains(&Self::get_root_node()) {
            self.copy_root_shell_from(other, true);
            return;
        }

        let copied = other.copy_subset_nodes(
            Self::get_root_node(),
            None,
            &complete_subtree,
            &partial_subtree,
        );
        self.nodes.clear();
        for (id, node) in copied {
            self.set_copied_node(id, node);
        }
        self.refresh_max_unique_id();
    }

    /// VTK: `vtkDataAssembly::AddDataSetIndex`.
    pub fn add_data_set_index(&mut self, id: i32, dataset_index: u32) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        if node.dataset_indices.contains(&dataset_index) {
            return true;
        }
        node.dataset_indices.push(dataset_index);
        self.modified();
        true
    }

    /// VTK: `vtkDataAssembly::AddDataSetIndices`.
    pub fn add_data_set_indices(&mut self, id: i32, dataset_indices: &[u32]) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        let mut modified = false;
        for &dataset_index in dataset_indices {
            if !node.dataset_indices.contains(&dataset_index) {
                node.dataset_indices.push(dataset_index);
                modified = true;
            }
        }
        if modified {
            self.modified();
        }
        modified
    }

    /// VTK: `vtkDataAssembly::AddDataSetIndexRange`.
    pub fn add_data_set_index_range(&mut self, id: i32, index_start: u32, count: i32) -> bool {
        let indices: Vec<u32> = (0..count)
            .map(|offset| index_start.wrapping_add(offset as u32))
            .collect();
        self.add_data_set_indices(id, &indices)
    }

    /// VTK: `vtkDataAssembly::RemoveDataSetIndex`.
    pub fn remove_data_set_index(&mut self, id: i32, dataset_index: u32) -> bool {
        let Some(node) = self.node_mut(id) else {
            return false;
        };
        let Some(position) = node
            .dataset_indices
            .iter()
            .position(|&index| index == dataset_index)
        else {
            return false;
        };
        node.dataset_indices.remove(position);
        self.modified();
        true
    }

    /// VTK: `vtkDataAssembly::RemoveAllDataSetIndices`.
    pub fn remove_all_data_set_indices(&mut self, id: i32, traverse_subtree: bool) -> bool {
        if self.node(id).is_none() {
            return false;
        }

        let ids = if traverse_subtree {
            self.traverse_ids(id, TraversalOrder::DepthFirst)
        } else {
            vec![id]
        };
        let mut modified = false;
        for node_id in ids {
            if let Some(node) = self.node_mut(node_id) {
                if !node.dataset_indices.is_empty() {
                    node.dataset_indices.clear();
                    modified = true;
                }
            }
        }
        if modified {
            self.modified();
        }
        modified
    }

    /// VTK: `vtkDataAssembly::FindFirstNodeWithName`.
    pub fn find_first_node_with_name(
        &self,
        name: Option<&str>,
        traversal_order: TraversalOrder,
    ) -> i32 {
        self.traverse_ids(Self::get_root_node(), traversal_order)
            .into_iter()
            .find(|&id| self.get_node_name(id) == name)
            .unwrap_or(-1)
    }

    /// VTK: `vtkDataAssembly::FindNodesWithName`.
    pub fn find_nodes_with_name(
        &self,
        name: Option<&str>,
        traversal_order: TraversalOrder,
    ) -> Vec<i32> {
        self.traverse_ids(Self::get_root_node(), traversal_order)
            .into_iter()
            .filter(|&id| self.get_node_name(id) == name)
            .collect()
    }

    /// VTK: `vtkDataAssembly::GetChildNodes`.
    pub fn get_child_nodes(
        &self,
        parent: i32,
        traverse_subtree: bool,
        traversal_order: TraversalOrder,
    ) -> Vec<i32> {
        if self.node(parent).is_none() {
            return Vec::new();
        }
        if traverse_subtree {
            self.traverse_ids(parent, traversal_order)
                .into_iter()
                .filter(|&id| id != parent)
                .collect()
        } else {
            self.node(parent)
                .map(|node| node.children.clone())
                .unwrap_or_default()
        }
    }

    /// VTK: `vtkDataAssembly::GetDataSetIndices`.
    pub fn get_data_set_indices(
        &self,
        id: i32,
        traverse_subtree: bool,
        traversal_order: TraversalOrder,
    ) -> Vec<u32> {
        self.get_data_set_indices_from_nodes(&[id], traverse_subtree, traversal_order)
    }

    /// VTK: `vtkDataAssembly::GetDataSetIndices`.
    pub fn get_data_set_indices_from_nodes(
        &self,
        ids: &[i32],
        traverse_subtree: bool,
        traversal_order: TraversalOrder,
    ) -> Vec<u32> {
        let mut indices = Vec::new();
        for &id in ids {
            let node_ids = if traverse_subtree {
                self.traverse_ids(id, traversal_order)
            } else if self.node(id).is_some() {
                vec![id]
            } else {
                Vec::new()
            };
            for node_id in node_ids {
                if let Some(node) = self.node(node_id) {
                    indices.extend(node.dataset_indices.iter().copied());
                }
            }
        }
        unique_preserve_order(indices)
    }

    /// VTK: `vtkDataAssembly::SelectNodes`.
    pub fn select_nodes(
        &self,
        path_queries: &[String],
        traversal_order: TraversalOrder,
    ) -> Vec<i32> {
        let mut selected = std::collections::BTreeSet::new();
        for query in path_queries {
            if query.is_empty() {
                continue;
            }
            selected.extend(self.select_nodes_for_query(query));
        }
        self.traverse_ids(Self::get_root_node(), traversal_order)
            .into_iter()
            .filter(|id| selected.contains(id))
            .collect()
    }

    /// VTK: `vtkDataAssembly::Visit`.
    pub fn visit(
        &self,
        id: i32,
        visitor: Option<&mut dyn DataAssemblyVisitor>,
        traversal_order: TraversalOrder,
    ) -> bool {
        let Some(visitor) = visitor else {
            return false;
        };
        if self.node(id).is_none() {
            return false;
        }

        match traversal_order {
            TraversalOrder::DepthFirst => {
                self.visit_depth_first(id, visitor, traversal_order);
            }
            TraversalOrder::BreadthFirst => {
                self.visit_breadth_first(id, visitor, traversal_order);
            }
        }
        true
    }

    /// VTK: `vtkDataAssembly::RemapDataSetIndices`.
    pub fn remap_data_set_indices(
        &mut self,
        mapping: &std::collections::BTreeMap<u32, u32>,
        remove_unmapped: bool,
    ) -> bool {
        let mut modified = false;
        for node in self.nodes.iter_mut().filter_map(Option::as_mut) {
            let mut index = 0;
            while index < node.dataset_indices.len() {
                let old_id = node.dataset_indices[index];
                if let Some(&new_id) = mapping.get(&old_id) {
                    if new_id != old_id {
                        node.dataset_indices[index] = new_id;
                        modified = true;
                    }
                    index += 1;
                } else if remove_unmapped {
                    node.dataset_indices.remove(index);
                    modified = true;
                } else {
                    index += 1;
                }
            }
        }
        if modified {
            self.modified();
        }
        modified
    }

    /// VTK: `vtkDataAssembly::GetNumberOfChildren`.
    pub fn get_number_of_children(&self, parent: i32) -> i32 {
        self.node(parent)
            .map(|node| i32::try_from(node.children.len()).unwrap_or(i32::MAX))
            .unwrap_or(0)
    }

    /// VTK: `vtkDataAssembly::GetChild`.
    pub fn get_child(&self, parent: i32, index: i32) -> i32 {
        let Some(index) = usize_from_id(index) else {
            return -1;
        };
        self.node(parent)
            .and_then(|node| node.children.get(index))
            .copied()
            .unwrap_or(-1)
    }

    /// VTK: `vtkDataAssembly::GetChildIndex`.
    pub fn get_child_index(&self, parent: i32, child: i32) -> i32 {
        self.node(parent)
            .and_then(|node| node.children.iter().position(|&id| id == child))
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1)
    }

    /// VTK: `vtkDataAssembly::GetParent`.
    pub fn get_parent(&self, id: i32) -> i32 {
        self.node(id).and_then(|node| node.parent).unwrap_or(-1)
    }

    /// VTK: `vtkDataAssembly::HasAttribute`.
    pub fn has_attribute(&self, id: i32, name: Option<&str>) -> bool {
        let Some(name) = name else {
            return false;
        };
        self.node(id)
            .and_then(|node| node.attribute(name))
            .is_some()
    }

    /// VTK: `vtkDataAssembly::SetAttribute`.
    pub fn set_attribute(&mut self, id: i32, name: Option<&str>, value: Option<&str>) {
        if let (Some(name), Some(value), Some(node)) = (name, value, self.node_mut(id)) {
            node.set_attribute(name, value);
        }
        self.modified();
    }

    /// VTK: `vtkDataAssembly::SetAttribute`.
    pub fn set_attribute_int(&mut self, id: i32, name: Option<&str>, value: i32) {
        self.set_attribute(id, name, Some(&value.to_string()));
    }

    /// VTK: `vtkDataAssembly::SetAttribute`.
    pub fn set_attribute_unsigned(&mut self, id: i32, name: Option<&str>, value: u32) {
        self.set_attribute(id, name, Some(&value.to_string()));
    }

    /// VTK: `vtkDataAssembly::SetAttribute`.
    pub fn set_attribute_id(&mut self, id: i32, name: Option<&str>, value: i64) {
        self.set_attribute(id, name, Some(&value.to_string()));
    }

    /// VTK: `vtkDataAssembly::GetAttribute`.
    pub fn get_attribute(&self, id: i32, name: Option<&str>) -> Option<&str> {
        let name = name?;
        self.node(id).and_then(|node| node.attribute(name))
    }

    /// VTK: `vtkDataAssembly::GetAttribute`.
    pub fn get_attribute_int(&self, id: i32, name: Option<&str>) -> Option<i32> {
        self.get_attribute(id, name)
            .map(|value| value.parse::<i32>().unwrap_or(0))
    }

    /// VTK: `vtkDataAssembly::GetAttribute`.
    pub fn get_attribute_unsigned(&self, id: i32, name: Option<&str>) -> Option<u32> {
        self.get_attribute(id, name)
            .map(|value| value.parse::<u32>().unwrap_or(0))
    }

    /// VTK: `vtkDataAssembly::GetAttribute`.
    pub fn get_attribute_id(&self, id: i32, name: Option<&str>) -> Option<i64> {
        self.get_attribute(id, name)
            .map(|value| value.parse::<i64>().unwrap_or(0))
    }

    /// VTK: `vtkDataAssembly::GetAttributeOrDefault`.
    pub fn get_attribute_or_default<'a>(
        &'a self,
        id: i32,
        name: Option<&str>,
        default_value: &'a str,
    ) -> &'a str {
        self.get_attribute(id, name).unwrap_or(default_value)
    }

    /// VTK: `vtkDataAssembly::GetAttributeOrDefault`.
    pub fn get_attribute_or_default_int(
        &self,
        id: i32,
        name: Option<&str>,
        default_value: i32,
    ) -> i32 {
        self.get_attribute(id, name)
            .map(|value| value.parse::<i32>().unwrap_or(default_value))
            .unwrap_or(default_value)
    }

    /// VTK: `vtkDataAssembly::GetAttributeOrDefault`.
    pub fn get_attribute_or_default_unsigned(
        &self,
        id: i32,
        name: Option<&str>,
        default_value: u32,
    ) -> u32 {
        self.get_attribute(id, name)
            .map(|value| value.parse::<u32>().unwrap_or(default_value))
            .unwrap_or(default_value)
    }

    /// VTK: `vtkDataAssembly::GetAttributeOrDefault`.
    pub fn get_attribute_or_default_id(
        &self,
        id: i32,
        name: Option<&str>,
        default_value: i64,
    ) -> i64 {
        self.get_attribute(id, name)
            .map(|value| value.parse::<i64>().unwrap_or(default_value))
            .unwrap_or(default_value)
    }

    /// VTK: `vtkDataAssembly::IsNodeNameValid`.
    pub fn is_node_name_valid(name: Option<&str>) -> bool {
        let Some(name) = name else {
            return false;
        };
        let Some(first) = name.as_bytes().first().copied() else {
            return false;
        };
        if Self::is_node_name_reserved(name) {
            return false;
        }
        if !is_name_start(first) {
            return false;
        }
        name.bytes().all(is_name_char)
    }

    /// VTK: `vtkDataAssembly::MakeValidNodeName`.
    pub fn make_valid_node_name(name: Option<&str>) -> String {
        let Some(name) = name else {
            return String::new();
        };
        if name.is_empty() || Self::is_node_name_reserved(name) {
            return String::new();
        }

        let mut result = String::with_capacity(name.len());
        for byte in name.bytes().filter(|byte| is_name_char(*byte)) {
            result.push(char::from(byte));
        }

        if result
            .as_bytes()
            .first()
            .is_none_or(|first| !is_name_start(*first))
        {
            result.insert(0, '_');
        }
        result
    }

    /// VTK: `vtkDataAssembly::IsNodeNameReserved`.
    pub fn is_node_name_reserved(name: &str) -> bool {
        let bytes = name.as_bytes();
        let dataset = DATASET_NODE_NAME.as_bytes();
        bytes.len() > 2
            && bytes[0] == dataset[0]
            && bytes[1] == dataset[1]
            && bytes[2..] == dataset[2..]
    }

    fn modified(&mut self) {
        self.modified_time = self.modified_time.saturating_add(1);
    }

    fn node(&self, id: i32) -> Option<&DataAssemblyNode> {
        usize_from_id(id)
            .and_then(|index| self.nodes.get(index))
            .and_then(Option::as_ref)
    }

    fn node_mut(&mut self, id: i32) -> Option<&mut DataAssemblyNode> {
        usize_from_id(id)
            .and_then(|index| self.nodes.get_mut(index))
            .and_then(Option::as_mut)
    }

    fn append_node(&mut self, parent: i32, name: &str) -> i32 {
        let child = self.max_unique_id.saturating_add(1);
        let child_index = usize_from_id(child).expect("positive node id must fit usize");
        if child_index >= self.nodes.len() {
            self.nodes.resize_with(child_index + 1, || None);
        }
        self.nodes[child_index] = Some(DataAssemblyNode {
            name: name.to_string(),
            parent: Some(parent),
            children: Vec::new(),
            dataset_indices: Vec::new(),
            attributes: Vec::new(),
        });
        self.node_mut(parent)
            .expect("validated parent node")
            .children
            .push(child);
        self.max_unique_id = child;
        child
    }

    fn remove_subtree(&mut self, id: i32) {
        let Some(index) = usize_from_id(id) else {
            return;
        };
        if index >= self.nodes.len() {
            return;
        }
        if let Some(node) = self.nodes[index].take() {
            for child in node.children {
                self.remove_subtree(child);
            }
        }
    }

    fn max_data_set_index(&self) -> Option<u32> {
        self.nodes
            .iter()
            .filter_map(Option::as_ref)
            .flat_map(|node| node.dataset_indices.iter().copied())
            .max()
    }

    fn copy_root_shell_from(&mut self, other: &Self, include_dataset_indices: bool) {
        let Some(source) = other.node(Self::get_root_node()) else {
            return;
        };
        let Some(root) = self.node_mut(Self::get_root_node()) else {
            return;
        };
        root.name.clone_from(&source.name);
        root.attributes.clone_from(&source.attributes);
        root.children.clear();
        if include_dataset_indices {
            root.dataset_indices.clone_from(&source.dataset_indices);
        } else {
            root.dataset_indices.clear();
        }
        self.max_unique_id = ROOT_NODE_ID;
    }

    fn copy_subtree_nodes(
        &self,
        id: i32,
        parent: Option<i32>,
        node_offset: i32,
        dataset_offset: u32,
    ) -> Vec<(i32, DataAssemblyNode)> {
        let Some(node) = self.node(id) else {
            return Vec::new();
        };
        let new_id = id.saturating_add(node_offset);
        let mut copied = DataAssemblyNode {
            name: node.name.clone(),
            parent,
            children: node
                .children
                .iter()
                .map(|child| child.saturating_add(node_offset))
                .collect(),
            dataset_indices: node
                .dataset_indices
                .iter()
                .map(|index| index.saturating_add(dataset_offset))
                .collect(),
            attributes: node.attributes.clone(),
        };
        let children = std::mem::take(&mut copied.children);
        let mut nodes = vec![(
            new_id,
            DataAssemblyNode {
                children: children.clone(),
                ..copied
            },
        )];
        for (&child, &new_child) in node.children.iter().zip(children.iter()) {
            nodes.extend(self.copy_subtree_nodes(child, Some(new_id), node_offset, dataset_offset));
            debug_assert_eq!(new_child, child.saturating_add(node_offset));
        }
        nodes
    }

    fn copy_subset_nodes(
        &self,
        id: i32,
        parent: Option<i32>,
        complete_subtree: &std::collections::BTreeSet<i32>,
        partial_subtree: &std::collections::BTreeSet<i32>,
    ) -> Vec<(i32, DataAssemblyNode)> {
        if complete_subtree.contains(&id) {
            return self.copy_subtree_nodes(id, parent, 0, 0);
        }

        let Some(node) = self.node(id) else {
            return Vec::new();
        };
        let children: Vec<i32> = node
            .children
            .iter()
            .copied()
            .filter(|child| complete_subtree.contains(child) || partial_subtree.contains(child))
            .collect();
        let mut nodes = vec![(
            id,
            DataAssemblyNode {
                name: node.name.clone(),
                parent,
                children: children.clone(),
                dataset_indices: node.dataset_indices.clone(),
                attributes: node.attributes.clone(),
            },
        )];
        for child in children {
            nodes.extend(self.copy_subset_nodes(
                child,
                Some(id),
                complete_subtree,
                partial_subtree,
            ));
        }
        nodes
    }

    fn set_copied_node(&mut self, id: i32, node: DataAssemblyNode) {
        let index = usize_from_id(id).expect("copied node id must fit usize");
        if index >= self.nodes.len() {
            self.nodes.resize_with(index + 1, || None);
        }
        self.nodes[index] = Some(node);
    }

    fn refresh_max_unique_id(&mut self) {
        self.max_unique_id = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(id, node)| node.as_ref().map(|_| id))
            .filter_map(|id| i32::try_from(id).ok())
            .max()
            .unwrap_or(ROOT_NODE_ID);
    }

    fn serialize_node(&self, id: i32, indent: &str, depth: usize, xml: &mut String) {
        let Some(node) = self.node(id) else {
            return;
        };
        for _ in 0..depth {
            xml.push_str(indent);
        }
        xml.push('<');
        xml.push_str(&node.name);
        xml.push_str(" id=\"");
        xml.push_str(&id.to_string());
        xml.push('"');
        if id == Self::get_root_node() {
            xml.push_str(" type=\"vtkDataAssembly\" version=\"1.0\"");
        }
        for (name, value) in &node.attributes {
            if name == "id"
                || (id == Self::get_root_node() && (name == "type" || name == "version"))
            {
                continue;
            }
            xml.push(' ');
            xml.push_str(name);
            xml.push_str("=\"");
            xml.push_str(&xml_escape_attribute(value));
            xml.push('"');
        }
        if node.children.is_empty() && node.dataset_indices.is_empty() {
            xml.push_str(" />\n");
            return;
        }
        xml.push_str(">\n");
        for dataset_index in &node.dataset_indices {
            for _ in 0..(depth + 1) {
                xml.push_str(indent);
            }
            xml.push('<');
            xml.push_str(DATASET_NODE_NAME);
            xml.push_str(" id=\"");
            xml.push_str(&dataset_index.to_string());
            xml.push_str("\" />\n");
        }
        for &child in &node.children {
            self.serialize_node(child, indent, depth + 1, xml);
        }
        for _ in 0..depth {
            xml.push_str(indent);
        }
        xml.push_str("</");
        xml.push_str(&node.name);
        xml.push_str(">\n");
    }

    fn child_with_name(&self, parent: i32, name: &str) -> Option<i32> {
        self.node(parent)?.children.iter().copied().find(|&child| {
            self.node(child)
                .map(|node| node.name.as_str() == name)
                .unwrap_or(false)
        })
    }

    fn first_node_by_pugi_path(&self, path: &str) -> Option<i32> {
        let mut node = Self::get_root_node();
        let mut segments = split_path_skip_empty(path);
        if path.starts_with('/') {
            if segments
                .first()
                .is_some_and(|name| self.get_node_name(node) == Some(*name))
            {
                segments.remove(0);
            } else if !segments.is_empty() {
                return None;
            }
        }

        for name in segments {
            if name == "." {
                continue;
            }
            if name == ".." {
                node = self.get_parent(node);
                if node == -1 {
                    return None;
                }
                continue;
            }
            node = self.child_with_name(node, name)?;
        }
        Some(node)
    }

    fn select_nodes_for_query(&self, query: &str) -> Vec<i32> {
        if query == "/" {
            return vec![Self::get_root_node()];
        }

        let select_children = query.len() > 1 && query.ends_with('/');
        let query = if select_children {
            query.trim_end_matches('/')
        } else {
            query
        };

        let mut matches = if let Some(rest) = query.strip_prefix("//") {
            let segments = split_query_path(rest);
            self.select_descendant_path(&segments)
        } else if let Some(rest) = query.strip_prefix('/') {
            let segments = split_query_path(rest);
            self.select_absolute_path(&segments)
        } else {
            Vec::new()
        };

        if select_children {
            matches = matches
                .into_iter()
                .flat_map(|id| {
                    self.node(id)
                        .map(|node| node.children.clone())
                        .unwrap_or_default()
                })
                .collect();
        }
        matches
    }

    fn select_absolute_path(&self, segments: &[String]) -> Vec<i32> {
        let Some(first) = segments.first() else {
            return vec![Self::get_root_node()];
        };
        let mut matches: Vec<i32> = self
            .node(Self::get_root_node())
            .map(|root| root.children.clone())
            .unwrap_or_default()
            .into_iter()
            .filter(|&id| self.get_node_name(id) == Some(first.as_str()))
            .collect();
        for segment in &segments[1..] {
            matches = self.select_named_children(&matches, segment);
        }
        matches
    }

    fn select_descendant_path(&self, segments: &[String]) -> Vec<i32> {
        let Some(first) = segments.first() else {
            return Vec::new();
        };
        let mut matches: Vec<i32> = self
            .traverse_ids(Self::get_root_node(), TraversalOrder::DepthFirst)
            .into_iter()
            .filter(|&id| id != Self::get_root_node())
            .filter(|&id| self.get_node_name(id) == Some(first.as_str()))
            .collect();
        for segment in &segments[1..] {
            matches = self.select_named_children(&matches, segment);
        }
        matches
    }

    fn select_named_children(&self, parents: &[i32], name: &str) -> Vec<i32> {
        parents
            .iter()
            .flat_map(|&parent| {
                self.node(parent)
                    .map(|node| node.children.clone())
                    .unwrap_or_default()
            })
            .filter(|&id| self.get_node_name(id) == Some(name))
            .collect()
    }

    fn get_node_path_components(&self, id: i32) -> Vec<&str> {
        if self.node(id).is_none() {
            return Vec::new();
        }

        let mut names = Vec::new();
        let mut current = Some(id);
        while let Some(node_id) = current {
            let Some(node) = self.node(node_id) else {
                return Vec::new();
            };
            names.push(node.name.as_str());
            current = node.parent;
        }
        names.reverse();
        names
    }

    fn visit_context(
        &self,
        current_node: i32,
        traversal_order: TraversalOrder,
    ) -> DataAssemblyVisitContext<'_> {
        DataAssemblyVisitContext {
            assembly: self,
            current_node,
            traversal_order,
        }
    }

    fn visit_depth_first(
        &self,
        id: i32,
        visitor: &mut dyn DataAssemblyVisitor,
        traversal_order: TraversalOrder,
    ) {
        let Some(node) = self.node(id) else {
            return;
        };
        let context = self.visit_context(id, traversal_order);
        visitor.visit(id, &context);

        if visitor.get_traverse_subtree(id, &context) {
            visitor.begin_sub_tree(id, &context);
            for &child in &node.children {
                self.visit_depth_first(child, visitor, traversal_order);
            }
            let context = self.visit_context(id, traversal_order);
            visitor.end_sub_tree(id, &context);
        }
    }

    fn visit_breadth_first(
        &self,
        id: i32,
        visitor: &mut dyn DataAssemblyVisitor,
        traversal_order: TraversalOrder,
    ) {
        let mut queue = std::collections::VecDeque::from([id]);
        let context = self.visit_context(id, traversal_order);
        visitor.visit(id, &context);

        while let Some(node_id) = queue.pop_front() {
            let Some(node) = self.node(node_id) else {
                continue;
            };
            let context = self.visit_context(node_id, traversal_order);
            if visitor.get_traverse_subtree(node_id, &context) {
                visitor.begin_sub_tree(node_id, &context);
                for &child in &node.children {
                    let child_context = self.visit_context(child, traversal_order);
                    visitor.visit(child, &child_context);
                    queue.push_back(child);
                }
                let context = self.visit_context(node_id, traversal_order);
                visitor.end_sub_tree(node_id, &context);
            }
        }
    }

    fn traverse_ids(&self, root: i32, traversal_order: TraversalOrder) -> Vec<i32> {
        if self.node(root).is_none() {
            return Vec::new();
        }
        match traversal_order {
            TraversalOrder::DepthFirst => {
                let mut ids = Vec::new();
                self.traverse_depth_first(root, &mut ids);
                ids
            }
            TraversalOrder::BreadthFirst => self.traverse_breadth_first(root),
        }
    }

    fn traverse_depth_first(&self, id: i32, ids: &mut Vec<i32>) {
        let Some(node) = self.node(id) else {
            return;
        };
        ids.push(id);
        for &child in &node.children {
            self.traverse_depth_first(child, ids);
        }
    }

    fn traverse_breadth_first(&self, root: i32) -> Vec<i32> {
        let mut ids = Vec::new();
        let mut queue = std::collections::VecDeque::from([root]);
        while let Some(id) = queue.pop_front() {
            let Some(node) = self.node(id) else {
                continue;
            };
            ids.push(id);
            queue.extend(node.children.iter().copied());
        }
        ids
    }
}

impl Default for DataAssembly {
    fn default() -> Self {
        Self::new()
    }
}

fn is_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-')
}

fn split_path_preserve_empty(path: &str) -> Vec<&str> {
    if path.is_empty() {
        Vec::new()
    } else {
        path.split('/').collect()
    }
}

fn split_path_skip_empty(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn split_query_path(path: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut segment = String::new();
    let mut escaped = false;
    for character in path.chars() {
        if escaped {
            segment.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '/' {
            if !segment.is_empty() {
                segments.push(std::mem::take(&mut segment));
            }
        } else {
            segment.push(character);
        }
    }
    if escaped {
        segment.push('\\');
    }
    if !segment.is_empty() {
        segments.push(segment);
    }
    segments
}

fn xml_escape_attribute(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn unique_preserve_order(indices: Vec<u32>) -> Vec<u32> {
    let mut unique = Vec::with_capacity(indices.len());
    for index in indices {
        if !unique.contains(&index) {
            unique.push(index);
        }
    }
    unique
}
