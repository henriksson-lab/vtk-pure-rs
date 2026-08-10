use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use crate::common::core::{HashCombiner, Object, StringToken, VtkIdType, VtkMTimeType};

const VTK_DBG_MAX_HASHES: usize = 1024;

/// VTK: `vtkCellGridSidesCache::Side`.
#[derive(Debug, Clone)]
pub struct CellGridSide {
    pub cell_type: StringToken,
    pub side_shape: StringToken,
    pub dof: VtkIdType,
    pub side_id: i32,
}

impl PartialEq for CellGridSide {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for CellGridSide {}

impl PartialOrd for CellGridSide {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellGridSide {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cell_type
            .cmp(&other.cell_type)
            .then_with(|| self.dof.cmp(&other.dof))
            .then_with(|| self.side_id.cmp(&other.side_id))
    }
}

/// VTK: `vtkCellGridSidesCache::Entry`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CellGridSidesCacheEntry {
    pub sides: BTreeSet<CellGridSide>,
}

/// VTK: `vtkCellGridSidesCache`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellGridSidesCache {
    object: Object,
    hashes: HashMap<usize, CellGridSidesCacheEntry>,
}

impl CellGridSidesCache {
    /// VTK: `vtkCellGridSidesCache::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCellGridSidesCache"),
            hashes: HashMap::new(),
        }
    }

    /// VTK: `vtkCellGridSidesCache::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = format!("Hashes: {} entries\n", self.hashes.len());
        let mut num_printed = 0;
        for (hash, entry) in &self.hashes {
            result.push_str(&format!("  {hash:x} ({})\n", entry.sides.len()));
            for side in &entry.sides {
                result.push_str(&format!(
                    "    {} {} start id {} side {}\n",
                    side.cell_type.data(),
                    side.side_shape.data(),
                    side.dof,
                    side.side_id
                ));
            }
            num_printed += 1;
            if num_printed > VTK_DBG_MAX_HASHES {
                if self.hashes.len() > num_printed {
                    result.push_str(&format!(
                        "  ... and {} more.\n",
                        self.hashes.len() - num_printed
                    ));
                }
                break;
            }
        }
        result
    }

    /// VTK: `vtkCellGridSidesCache::GetHashes`.
    pub fn get_hashes(&mut self) -> &mut HashMap<usize, CellGridSidesCacheEntry> {
        &mut self.hashes
    }

    /// VTK: `vtkCellGridSidesCache::HashSide`.
    pub fn hash_side<T>(&self, shape: StringToken, conn: &[T]) -> usize
    where
        T: CellGridSideConnectivityValue,
    {
        let nn = conn.len();
        if nn == 0 {
            return 0;
        }

        let mut ss = 0;
        let mut smin = conn[0];
        for jj in 1..nn {
            if conn[jj] < smin {
                smin = conn[jj];
                ss = jj;
            }
        }

        let forward = conn[(ss + 1) % nn] > conn[(ss + nn - 1) % nn];
        let combiner = HashCombiner;
        let mut hashed_value = nn;
        combiner.combine_usize(&mut hashed_value, shape.get_id() as usize);

        if forward {
            for ii in 0..nn {
                combiner.combine_usize(&mut hashed_value, conn[(ss + ii) % nn].hash_token());
            }
        } else {
            for ii in 0..nn {
                combiner.combine_usize(&mut hashed_value, conn[(ss + nn - ii) % nn].hash_token());
            }
        }
        hashed_value
    }

    /// VTK: `vtkCellGridSidesCache::AddSide`.
    pub fn add_side<T>(
        &mut self,
        cell_type: StringToken,
        cell: VtkIdType,
        side: i32,
        shape: StringToken,
        conn: &[T],
    ) where
        T: CellGridSideConnectivityValue,
    {
        let hashed_value = self.hash_side(shape, conn);
        self.hashes
            .entry(hashed_value)
            .or_default()
            .sides
            .insert(CellGridSide {
                cell_type,
                side_shape: shape,
                dof: cell,
                side_id: side,
            });
    }

    /// VTK: `vtkCellGridSidesCache::Initialize`.
    pub fn initialize(&mut self) {
        self.hashes.clear();
        self.modified();
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

impl Default for CellGridSidesCache {
    fn default() -> Self {
        Self::new()
    }
}

pub trait CellGridSideConnectivityValue: Copy + Ord {
    fn hash_token(self) -> usize;
}

macro_rules! impl_cell_grid_side_connectivity_value_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl CellGridSideConnectivityValue for $ty {
                fn hash_token(self) -> usize {
                    self as usize
                }
            }
        )*
    };
}

macro_rules! impl_cell_grid_side_connectivity_value_signed {
    ($($ty:ty),* $(,)?) => {
        $(
            impl CellGridSideConnectivityValue for $ty {
                fn hash_token(self) -> usize {
                    self as usize
                }
            }
        )*
    };
}

impl_cell_grid_side_connectivity_value_unsigned!(u8, u16, u32, u64, usize);
impl_cell_grid_side_connectivity_value_signed!(i8, i16, i32, i64, isize);
