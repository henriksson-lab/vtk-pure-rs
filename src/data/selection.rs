/// Type of content in a selection node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionContentType {
    /// Selection by global IDs.
    GlobalIds,
    /// Selection by pedigree IDs.
    PedigreeIds,
    /// Selection by indices (point or cell indices).
    Indices,
    /// Selection by frustum (6 planes).
    Frustum,
    /// Selection by value range on a named array.
    Thresholds,
    /// Selection by block index (for MultiBlock).
    Blocks,
}

/// Whether the selection applies to points or cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionFieldType {
    Point,
    Cell,
}

/// A single node in a selection, specifying what to select and how.
#[derive(Debug, Clone)]
pub struct SelectionNode {
    pub content_type: SelectionContentType,
    pub field_type: SelectionFieldType,
    /// The selection data — interpretation depends on content_type.
    /// For Indices: list of i64 indices.
    /// For Thresholds: [min, max] pairs.
    pub selection_list: Vec<f64>,
    /// Optional array name for Thresholds content type.
    pub array_name: Option<String>,
}

impl SelectionNode {
    /// Create a selection by point indices.
    pub fn from_point_indices(indices: Vec<i64>) -> Self {
        Self {
            content_type: SelectionContentType::Indices,
            field_type: SelectionFieldType::Point,
            selection_list: indices.into_iter().map(|i| i as f64).collect(),
            array_name: None,
        }
    }

    /// Create a selection by cell indices.
    pub fn from_cell_indices(indices: Vec<i64>) -> Self {
        Self {
            content_type: SelectionContentType::Indices,
            field_type: SelectionFieldType::Cell,
            selection_list: indices.into_iter().map(|i| i as f64).collect(),
            array_name: None,
        }
    }

    /// Create a threshold selection on a named array.
    pub fn from_threshold(
        array_name: &str,
        min: f64,
        max: f64,
        field_type: SelectionFieldType,
    ) -> Self {
        Self {
            content_type: SelectionContentType::Thresholds,
            field_type,
            selection_list: vec![min, max],
            array_name: Some(array_name.to_string()),
        }
    }
}

/// A selection consisting of one or more selection nodes.
#[derive(Debug, Clone, Default)]
pub struct Selection {
    nodes: Vec<SelectionNode>,
}

impl Selection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: SelectionNode) {
        self.nodes.push(node);
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn node(&self, idx: usize) -> Option<&SelectionNode> {
        self.nodes.get(idx)
    }

    pub fn nodes(&self) -> &[SelectionNode] {
        &self.nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_index_selection() {
        let mut sel = Selection::new();
        sel.add_node(SelectionNode::from_point_indices(vec![0, 5, 10]));
        assert_eq!(sel.num_nodes(), 1);
        let node = sel.node(0).unwrap();
        assert_eq!(node.content_type, SelectionContentType::Indices);
        assert_eq!(node.field_type, SelectionFieldType::Point);
        assert_eq!(node.selection_list.len(), 3);
    }

    #[test]
    fn threshold_selection() {
        let node =
            SelectionNode::from_threshold("temperature", 100.0, 200.0, SelectionFieldType::Cell);
        assert_eq!(node.content_type, SelectionContentType::Thresholds);
        assert_eq!(node.array_name.as_deref(), Some("temperature"));
        assert_eq!(node.selection_list, vec![100.0, 200.0]);
    }
}
