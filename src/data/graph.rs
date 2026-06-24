use crate::data::{DataSetAttributes, FieldData};

/// A graph data structure with vertices and edges.
///
/// Analogous to VTK's `vtkGraph` / `vtkMutableDirectedGraph` / `vtkMutableUndirectedGraph`.
/// Vertices and edges can carry associated data arrays via `DataSetAttributes`.
#[derive(Debug, Clone)]
pub struct Graph {
    /// Whether edges are directed.
    pub directed: bool,
    /// Edge list: `(source_vertex, target_vertex)`.
    edges: Vec<(usize, usize)>,
    /// Number of vertices.
    num_vertices: usize,
    /// Vertex data attributes.
    vertex_data: DataSetAttributes,
    /// Edge data attributes.
    edge_data: DataSetAttributes,
    /// Field data (metadata).
    field_data: FieldData,
}

impl Graph {
    /// Create an empty undirected graph.
    pub fn new_undirected() -> Self {
        Self {
            directed: false,
            edges: Vec::new(),
            num_vertices: 0,
            vertex_data: DataSetAttributes::new(),
            edge_data: DataSetAttributes::new(),
            field_data: FieldData::new(),
        }
    }

    /// Create an empty directed graph.
    pub fn new_directed() -> Self {
        Self {
            directed: true,
            edges: Vec::new(),
            num_vertices: 0,
            vertex_data: DataSetAttributes::new(),
            edge_data: DataSetAttributes::new(),
            field_data: FieldData::new(),
        }
    }

    /// Add a vertex and return its index.
    pub fn add_vertex(&mut self) -> usize {
        let id = self.num_vertices;
        self.num_vertices += 1;
        id
    }

    /// Add N vertices and return the index of the first new vertex.
    pub fn add_vertices(&mut self, count: usize) -> usize {
        let first = self.num_vertices;
        self.num_vertices = self
            .num_vertices
            .checked_add(count)
            .expect("vertex count overflow");
        first
    }

    /// Add an edge between two vertices and return the edge index.
    pub fn add_edge(&mut self, source: usize, target: usize) -> usize {
        assert!(source < self.num_vertices, "source vertex out of range");
        assert!(target < self.num_vertices, "target vertex out of range");
        let id = self.edges.len();
        self.edges.push((source, target));
        id
    }

    /// Number of vertices.
    pub fn num_vertices(&self) -> usize {
        self.num_vertices
    }

    /// Number of edges.
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    /// Get edge by index: `(source, target)`.
    pub fn edge(&self, idx: usize) -> (usize, usize) {
        self.edges[idx]
    }

    /// Iterate over all edges as `(source, target)`.
    pub fn edges(&self) -> &[(usize, usize)] {
        &self.edges
    }

    /// Get the degree (number of incident edges) of a vertex.
    pub fn degree(&self, vertex: usize) -> usize {
        if self.directed {
            self.out_degree(vertex) + self.in_degree(vertex)
        } else {
            self.out_degree(vertex)
        }
    }

    /// Get the out-degree of a vertex.
    pub fn out_degree(&self, vertex: usize) -> usize {
        assert!(vertex < self.num_vertices, "vertex out of range");
        self.edges
            .iter()
            .filter(|(s, t)| *s == vertex || (!self.directed && *t == vertex))
            .count()
    }

    /// Get the in-degree of a vertex.
    pub fn in_degree(&self, vertex: usize) -> usize {
        assert!(vertex < self.num_vertices, "vertex out of range");
        if self.directed {
            self.edges.iter().filter(|(_, t)| *t == vertex).count()
        } else {
            self.out_degree(vertex)
        }
    }

    /// Get all neighbors of a vertex.
    pub fn neighbors(&self, vertex: usize) -> Vec<usize> {
        assert!(vertex < self.num_vertices, "vertex out of range");
        let mut result = Vec::new();
        for &(s, t) in &self.edges {
            if s == vertex {
                result.push(t);
            } else if !self.directed && t == vertex {
                result.push(s);
            }
        }
        result
    }

    /// Get outgoing edges from a vertex (for directed graphs; all edges for undirected).
    pub fn out_edges(&self, vertex: usize) -> Vec<usize> {
        assert!(vertex < self.num_vertices, "vertex out of range");
        self.edges
            .iter()
            .enumerate()
            .filter(|(_, (s, t))| *s == vertex || (!self.directed && *t == vertex))
            .map(|(i, _)| i)
            .collect()
    }

    /// Build an adjacency list representation for efficient traversal.
    pub fn adjacency_list(&self) -> Vec<Vec<usize>> {
        let mut adj = vec![Vec::new(); self.num_vertices];
        for &(s, t) in &self.edges {
            if s < self.num_vertices {
                adj[s].push(t);
            }
            if !self.directed && t < self.num_vertices {
                adj[t].push(s);
            }
        }
        adj
    }

    pub fn vertex_data(&self) -> &DataSetAttributes {
        &self.vertex_data
    }

    pub fn vertex_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.vertex_data
    }

    pub fn edge_data(&self) -> &DataSetAttributes {
        &self.edge_data
    }

    pub fn edge_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.edge_data
    }

    pub fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    pub fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

/// A tree data structure (rooted, directed acyclic graph).
///
/// Analogous to VTK's `vtkTree`. Each node has at most one parent.
#[derive(Debug, Clone)]
pub struct Tree {
    /// Parent index for each node. Root node has parent == usize::MAX.
    parents: Vec<usize>,
    /// Children list for each node.
    children: Vec<Vec<usize>>,
    /// Vertex data attributes.
    vertex_data: DataSetAttributes,
    /// Edge data attributes.
    edge_data: DataSetAttributes,
}

impl Tree {
    /// Create a tree with a single root node. Returns the root index (0).
    pub fn new() -> Self {
        Self {
            parents: vec![usize::MAX],
            children: vec![Vec::new()],
            vertex_data: DataSetAttributes::new(),
            edge_data: DataSetAttributes::new(),
        }
    }

    /// Add a child node to the given parent. Returns the new node index.
    pub fn add_child(&mut self, parent: usize) -> usize {
        assert!(parent < self.parents.len(), "parent node out of range");
        let id = self.parents.len();
        self.parents.push(parent);
        self.children.push(Vec::new());
        self.children[parent].push(id);
        id
    }

    /// Number of nodes.
    pub fn num_nodes(&self) -> usize {
        self.parents.len()
    }

    /// Root node index (always 0).
    pub fn root(&self) -> usize {
        0
    }

    /// Parent of a node. Returns `None` for the root.
    pub fn parent(&self, node: usize) -> Option<usize> {
        assert!(node < self.parents.len(), "node out of range");
        let p = self.parents[node];
        if p == usize::MAX {
            None
        } else {
            Some(p)
        }
    }

    /// Children of a node.
    pub fn children(&self, node: usize) -> &[usize] {
        assert!(node < self.children.len(), "node out of range");
        &self.children[node]
    }

    /// Whether a node is a leaf (no children).
    pub fn is_leaf(&self, node: usize) -> bool {
        assert!(node < self.children.len(), "node out of range");
        self.children[node].is_empty()
    }

    /// Depth of a node (root = 0).
    pub fn depth(&self, node: usize) -> usize {
        assert!(node < self.parents.len(), "node out of range");
        let mut d = 0;
        let mut cur = node;
        while self.parents[cur] != usize::MAX {
            cur = self.parents[cur];
            d += 1;
        }
        d
    }

    /// Number of edges (always num_nodes - 1 for a tree).
    pub fn num_edges(&self) -> usize {
        if self.parents.is_empty() {
            0
        } else {
            self.parents.len() - 1
        }
    }

    /// Get all leaf nodes.
    pub fn leaves(&self) -> Vec<usize> {
        (0..self.num_nodes()).filter(|&n| self.is_leaf(n)).collect()
    }

    pub fn vertex_data(&self) -> &DataSetAttributes {
        &self.vertex_data
    }

    pub fn vertex_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.vertex_data
    }

    pub fn edge_data(&self) -> &DataSetAttributes {
        &self.edge_data
    }

    pub fn edge_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.edge_data
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undirected_graph() {
        let mut g = Graph::new_undirected();
        let v0 = g.add_vertex();
        let v1 = g.add_vertex();
        let v2 = g.add_vertex();
        g.add_edge(v0, v1);
        g.add_edge(v1, v2);

        assert_eq!(g.num_vertices(), 3);
        assert_eq!(g.num_edges(), 2);
        assert_eq!(g.degree(v1), 2);
        assert_eq!(g.neighbors(v0), vec![v1]);
        assert!(!g.directed);
    }

    #[test]
    fn directed_graph() {
        let mut g = Graph::new_directed();
        g.add_vertices(3);
        g.add_edge(0, 1);
        g.add_edge(0, 2);

        assert!(g.directed);
        assert_eq!(g.degree(0), 2);
        assert_eq!(g.out_degree(0), 2);
        assert_eq!(g.in_degree(0), 0);
        assert_eq!(g.degree(1), 1);
        assert_eq!(g.out_degree(1), 0);
        assert_eq!(g.in_degree(1), 1);
        assert_eq!(g.neighbors(0), vec![1, 2]);
    }

    #[test]
    fn directed_self_loop_counts_as_in_and_out_degree() {
        let mut g = Graph::new_directed();
        g.add_vertex();
        g.add_edge(0, 0);

        assert_eq!(g.out_degree(0), 1);
        assert_eq!(g.in_degree(0), 1);
        assert_eq!(g.degree(0), 2);
    }

    #[test]
    fn adjacency_list() {
        let mut g = Graph::new_undirected();
        g.add_vertices(4);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(2, 3);

        let adj = g.adjacency_list();
        assert_eq!(adj[0].len(), 2);
        assert_eq!(adj[3], vec![2]);
    }

    #[test]
    fn tree_basic() {
        let mut t = Tree::new();
        let c1 = t.add_child(0);
        let c2 = t.add_child(0);
        let c1_1 = t.add_child(c1);

        assert_eq!(t.num_nodes(), 4);
        assert_eq!(t.num_edges(), 3);
        assert_eq!(t.root(), 0);
        assert_eq!(t.parent(c1), Some(0));
        assert_eq!(t.parent(0), None);
        assert_eq!(t.children(0), &[c1, c2]);
        assert!(t.is_leaf(c2));
        assert!(!t.is_leaf(c1));
        assert_eq!(t.depth(c1_1), 2);
        assert_eq!(t.leaves(), vec![c2, c1_1]);
    }

    #[test]
    #[should_panic(expected = "target vertex out of range")]
    fn add_edge_rejects_invalid_target() {
        let mut g = Graph::new_undirected();
        g.add_vertex();
        g.add_edge(0, 1);
    }

    #[test]
    #[should_panic(expected = "parent node out of range")]
    fn add_child_rejects_invalid_parent() {
        let mut t = Tree::new();
        t.add_child(99);
    }
}
