use std::collections::BTreeMap;

use crate::common::core::{IdList, IdListCollection, VtkIdType};

type Triangle = [VtkIdType; 3];
type Edge = (VtkIdType, VtkIdType);

/// VTK: `vtkPolygonBuilder`.
#[derive(Debug, Default, Clone)]
pub struct PolygonBuilder {
    tris: BTreeMap<VtkIdType, Vec<Triangle>>,
    edge_counter: BTreeMap<Edge, usize>,
    edges: BTreeMap<VtkIdType, Vec<VtkIdType>>,
}

impl PolygonBuilder {
    /// VTK: `vtkPolygonBuilder::vtkPolygonBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// VTK: `vtkPolygonBuilder::InsertTriangle`.
    pub fn insert_triangle(&mut self, abc: Option<[VtkIdType; 3]>) {
        let Some(abc) = abc else {
            return;
        };

        if abc[0] == abc[1] || abc[0] == abc[2] || abc[1] == abc[2] {
            return;
        }

        let key = abc[0] + abc[1] + abc[2];
        let mut duplicate = false;
        let tris = self.tris.entry(key).or_default();
        for tri in tris.iter() {
            for i in 0..3 {
                let ta = tri[i % 3];
                let tb = tri[(i + 1) % 3];
                let tc = tri[(i + 2) % 3];
                if abc[0] == ta && abc[1] == tb && abc[2] == tc {
                    duplicate = true;
                    break;
                }
                if abc[2] == ta && abc[1] == tb && abc[0] == tc {
                    duplicate = true;
                    break;
                }
            }
            if duplicate {
                break;
            }
        }
        tris.push(abc);

        if duplicate {
            return;
        }

        for i in 0..3 {
            let edge = (abc[i], abc[(i + 1) % 3]);
            let inverse_edge = (abc[(i + 1) % 3], abc[i]);

            *self.edge_counter.entry(edge).or_insert(0) += 1;

            if self.edge_count(inverse_edge) == 0 {
                self.insert_edge(edge);
            } else if self.edge_count(edge) == 1 {
                self.erase_edge(inverse_edge);
            }
        }
    }

    /// VTK: `vtkPolygonBuilder::GetPolygons`.
    pub fn get_polygons(&mut self, polys: &mut IdListCollection) {
        polys.remove_all_items();

        if self.edge_len() < 3 {
            return;
        }

        while !self.edges.is_empty() {
            let mut poly = Box::new(IdList::new());
            let Some(mut edge) = self.first_edge() else {
                break;
            };
            let first_vtx = edge.0;

            loop {
                poly.insert_next_id(edge.0);
                let Some(next_edge) = self.first_edge_from(edge.1) else {
                    self.erase_edge(edge);
                    poly.reset();
                    break;
                };
                edge = next_edge;
                self.erase_edge(edge);
                if edge.0 == first_vtx {
                    break;
                }
            }

            if poly.get_number_of_ids() > 0 {
                polys.add_item(Box::into_raw(poly));
            }
        }

        self.reset();
    }

    /// VTK: `vtkPolygonBuilder::Reset`.
    pub fn reset(&mut self) {
        self.edge_counter.clear();
        self.edges.clear();
    }

    fn edge_count(&self, edge: Edge) -> usize {
        self.edge_counter.get(&edge).copied().unwrap_or(0)
    }

    fn insert_edge(&mut self, edge: Edge) {
        self.edges.entry(edge.0).or_default().push(edge.1);
    }

    fn erase_edge(&mut self, edge: Edge) -> bool {
        let Some(values) = self.edges.get_mut(&edge.0) else {
            return false;
        };
        let Some(index) = values.iter().position(|value| *value == edge.1) else {
            return false;
        };
        values.remove(index);
        if values.is_empty() {
            self.edges.remove(&edge.0);
        }
        true
    }

    fn first_edge(&self) -> Option<Edge> {
        self.edges
            .iter()
            .next()
            .and_then(|(&first, seconds)| seconds.first().map(|&second| (first, second)))
    }

    fn first_edge_from(&self, first: VtkIdType) -> Option<Edge> {
        self.edges
            .get(&first)
            .and_then(|seconds| seconds.first().map(|&second| (first, second)))
    }

    fn edge_len(&self) -> usize {
        self.edges.values().map(Vec::len).sum()
    }
}
