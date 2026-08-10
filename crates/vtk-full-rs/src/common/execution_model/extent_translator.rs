use std::sync::OnceLock;

use crate::common::{
    core::{InformationIntegerKey, Object, VtkMTimeType},
    execution_model::InformationIntegerRequestKey,
};

static DATA_SPLIT_MODE_KEY: OnceLock<usize> = OnceLock::new();
static UPDATE_SPLIT_MODE_KEY: OnceLock<usize> = OnceLock::new();

/// VTK: `vtkExtentTranslator::Modes::X_SLAB_MODE`.
pub const X_SLAB_MODE: i32 = 0;
/// VTK: `vtkExtentTranslator::Modes::Y_SLAB_MODE`.
pub const Y_SLAB_MODE: i32 = 1;
/// VTK: `vtkExtentTranslator::Modes::Z_SLAB_MODE`.
pub const Z_SLAB_MODE: i32 = 2;
/// VTK: `vtkExtentTranslator::Modes::BLOCK_MODE`.
pub const BLOCK_MODE: i32 = 3;

/// VTK: `vtkExtentTranslator`.
#[derive(Debug)]
pub struct ExtentTranslator {
    object: Object,
    piece: i32,
    number_of_pieces: i32,
    ghost_level: i32,
    extent: [i32; 6],
    whole_extent: [i32; 6],
    split_mode: i32,
    split_path: Vec<i32>,
}

impl ExtentTranslator {
    /// VTK: `vtkExtentTranslator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkExtentTranslator"),
            piece: 0,
            number_of_pieces: 0,
            ghost_level: 0,
            extent: [0, -1, 0, -1, 0, -1],
            whole_extent: [0, -1, 0, -1, 0, -1],
            split_mode: BLOCK_MODE,
            split_path: Vec::new(),
        }
    }

    /// VTK: `vtkExtentTranslator::DATA_SPLIT_MODE`.
    pub fn data_split_mode() -> &'static InformationIntegerKey {
        let key = *DATA_SPLIT_MODE_KEY.get_or_init(|| {
            InformationIntegerKey::make_key(Some("DATA_SPLIT_MODE"), Some("vtkExtentTranslator"))
                as usize
        });
        unsafe { &*(key as *const InformationIntegerKey) }
    }

    /// VTK: `vtkExtentTranslator::UPDATE_SPLIT_MODE`.
    pub fn update_split_mode() -> &'static InformationIntegerRequestKey {
        let key = *UPDATE_SPLIT_MODE_KEY.get_or_init(|| {
            let key = InformationIntegerRequestKey::make_key(
                Some("UPDATE_SPLIT_MODE"),
                Some("vtkExtentTranslator"),
            );
            unsafe {
                (*key).data_key =
                    Some(Self::data_split_mode() as *const InformationIntegerKey as usize);
            }
            key as usize
        });
        unsafe { &*(key as *const InformationIntegerRequestKey) }
    }

    /// VTK: `vtkExtentTranslator::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.object.get_object_description();
        output.push_str(&format!(
            "\nPiece: {}\nNumberOfPieces: {}\nGhostLevel: {}",
            self.piece, self.number_of_pieces, self.ghost_level
        ));
        output.push_str(&format!(
            "\nExtent: {}, {}, {}, {}, {}, {}",
            self.extent[0],
            self.extent[1],
            self.extent[2],
            self.extent[3],
            self.extent[4],
            self.extent[5]
        ));
        output.push_str(&format!(
            "\nWholeExtent: {}, {}, {}, {}, {}, {}",
            self.whole_extent[0],
            self.whole_extent[1],
            self.whole_extent[2],
            self.whole_extent[3],
            self.whole_extent[4],
            self.whole_extent[5]
        ));
        output.push_str("\nSplitMode: ");
        output.push_str(match self.split_mode {
            BLOCK_MODE => "Block",
            X_SLAB_MODE => "X Slab",
            Y_SLAB_MODE => "Y Slab",
            Z_SLAB_MODE => "Z Slab",
            _ => "Unknown",
        });
        output
    }

    /// VTK: `vtkExtentTranslator::SetWholeExtent`.
    pub fn set_whole_extent(&mut self, whole_extent: [i32; 6]) {
        self.whole_extent = whole_extent;
    }

    /// VTK: `vtkExtentTranslator::GetWholeExtent`.
    pub fn get_whole_extent(&self) -> [i32; 6] {
        self.whole_extent
    }

    /// VTK: `vtkExtentTranslator::SetExtent`.
    pub fn set_extent(&mut self, extent: [i32; 6]) {
        self.extent = extent;
    }

    /// VTK: `vtkExtentTranslator::GetExtent`.
    pub fn get_extent(&self) -> [i32; 6] {
        self.extent
    }

    /// VTK: `vtkExtentTranslator::SetPiece`.
    pub fn set_piece(&mut self, piece: i32) {
        self.piece = piece;
    }

    /// VTK: `vtkExtentTranslator::GetPiece`.
    pub fn get_piece(&self) -> i32 {
        self.piece
    }

    /// VTK: `vtkExtentTranslator::SetNumberOfPieces`.
    pub fn set_number_of_pieces(&mut self, number_of_pieces: i32) {
        self.number_of_pieces = number_of_pieces;
    }

    /// VTK: `vtkExtentTranslator::GetNumberOfPieces`.
    pub fn get_number_of_pieces(&self) -> i32 {
        self.number_of_pieces
    }

    /// VTK: `vtkExtentTranslator::SetGhostLevel`.
    pub fn set_ghost_level(&mut self, ghost_level: i32) {
        self.ghost_level = ghost_level;
    }

    /// VTK: `vtkExtentTranslator::GetGhostLevel`.
    pub fn get_ghost_level(&self) -> i32 {
        self.ghost_level
    }

    /// VTK: `vtkExtentTranslator::SetSplitModeToBlock`.
    pub fn set_split_mode_to_block(&mut self) {
        self.split_mode = BLOCK_MODE;
    }

    /// VTK: `vtkExtentTranslator::SetSplitModeToXSlab`.
    pub fn set_split_mode_to_x_slab(&mut self) {
        self.split_mode = X_SLAB_MODE;
    }

    /// VTK: `vtkExtentTranslator::SetSplitModeToYSlab`.
    pub fn set_split_mode_to_y_slab(&mut self) {
        self.split_mode = Y_SLAB_MODE;
    }

    /// VTK: `vtkExtentTranslator::SetSplitModeToZSlab`.
    pub fn set_split_mode_to_z_slab(&mut self) {
        self.split_mode = Z_SLAB_MODE;
    }

    /// VTK: `vtkExtentTranslator::GetSplitMode`.
    pub fn get_split_mode(&self) -> i32 {
        self.split_mode
    }

    /// VTK: `vtkExtentTranslator::SetSplitPath`.
    pub fn set_split_path(&mut self, split_path: &[i32]) {
        self.split_path.clear();
        self.split_path.extend_from_slice(split_path);
    }

    /// VTK: `vtkExtentTranslator::PieceToExtent`.
    pub fn piece_to_extent(&mut self) -> i32 {
        let mut result_extent = self.extent;
        let ret = self.piece_to_extent_thread_safe(
            self.piece,
            self.number_of_pieces,
            self.ghost_level,
            self.whole_extent,
            &mut result_extent,
            self.split_mode,
            0,
        );
        self.extent = result_extent;
        ret
    }

    /// VTK: `vtkExtentTranslator::PieceToExtentByPoints`.
    pub fn piece_to_extent_by_points(&mut self) -> i32 {
        let mut result_extent = self.extent;
        let ret = self.piece_to_extent_thread_safe(
            self.piece,
            self.number_of_pieces,
            self.ghost_level,
            self.whole_extent,
            &mut result_extent,
            self.split_mode,
            1,
        );
        self.extent = result_extent;
        ret
    }

    /// VTK: `vtkExtentTranslator::PieceToExtentThreadSafe`.
    pub fn piece_to_extent_thread_safe(
        &self,
        piece: i32,
        num_pieces: i32,
        ghost_level: i32,
        whole_extent: [i32; 6],
        result_extent: &mut [i32; 6],
        split_mode: i32,
        by_points: i32,
    ) -> i32 {
        *result_extent = whole_extent;
        let ret = if by_points != 0 {
            self.split_extent_by_points(piece, num_pieces, result_extent, split_mode)
        } else {
            self.split_extent(piece, num_pieces, result_extent, split_mode)
        };

        if ret == 0 {
            *result_extent = [0, -1, 0, -1, 0, -1];
            return 0;
        }

        if ghost_level > 0 {
            result_extent[0] -= ghost_level;
            result_extent[1] += ghost_level;
            result_extent[2] -= ghost_level;
            result_extent[3] += ghost_level;
            result_extent[4] -= ghost_level;
            result_extent[5] += ghost_level;

            result_extent[0] = result_extent[0].max(whole_extent[0]);
            result_extent[1] = result_extent[1].min(whole_extent[1]);
            result_extent[2] = result_extent[2].max(whole_extent[2]);
            result_extent[3] = result_extent[3].min(whole_extent[3]);
            result_extent[4] = result_extent[4].max(whole_extent[4]);
            result_extent[5] = result_extent[5].min(whole_extent[5]);
        }

        1
    }

    /// VTK: `vtkExtentTranslator::SplitExtent`.
    pub fn split_extent(
        &self,
        mut piece: i32,
        mut num_pieces: i32,
        ext: &mut [i32; 6],
        mut split_mode: i32,
    ) -> i32 {
        if piece >= num_pieces || piece < 0 {
            return 0;
        }

        let mut cnt = 0usize;
        while num_pieces > 1 {
            let size = [ext[1] - ext[0], ext[3] - ext[2], ext[5] - ext[4]];
            if cnt < self.split_path.len() {
                split_mode = self.split_path[cnt];
                cnt += 1;
            }

            let split_axis = choose_split_axis(size, split_mode);
            if split_axis < 0 {
                if piece == 0 {
                    num_pieces = 1;
                } else {
                    return 0;
                }
            } else {
                let split_axis = split_axis as usize;
                let num_pieces_in_first_half = num_pieces / 2;
                let mid = ((size[split_axis] as i64 * num_pieces_in_first_half as i64)
                    / num_pieces as i64)
                    + ext[split_axis * 2] as i64;
                let mid = mid as i32;
                if piece < num_pieces_in_first_half {
                    ext[split_axis * 2 + 1] = mid;
                    num_pieces = num_pieces_in_first_half;
                } else {
                    ext[split_axis * 2] = mid;
                    num_pieces -= num_pieces_in_first_half;
                    piece -= num_pieces_in_first_half;
                }
            }
        }

        1
    }

    /// VTK: `vtkExtentTranslator::SplitExtentByPoints`.
    pub fn split_extent_by_points(
        &self,
        mut piece: i32,
        mut num_pieces: i32,
        ext: &mut [i32; 6],
        split_mode: i32,
    ) -> i32 {
        while num_pieces > 1 {
            let size = [
                ext[1] - ext[0] + 1,
                ext[3] - ext[2] + 1,
                ext[5] - ext[4] + 1,
            ];
            let split_axis = choose_split_axis(size, split_mode);
            if split_axis < 0 {
                if piece == 0 {
                    num_pieces = 1;
                } else {
                    return 0;
                }
            } else {
                let split_axis = split_axis as usize;
                let num_pieces_in_first_half = num_pieces / 2;
                let mid = ((size[split_axis] as i64 * num_pieces_in_first_half as i64)
                    / num_pieces as i64)
                    + ext[split_axis * 2] as i64;
                let mid = mid as i32;
                if piece < num_pieces_in_first_half {
                    ext[split_axis * 2 + 1] = mid - 1;
                    num_pieces = num_pieces_in_first_half;
                } else {
                    ext[split_axis * 2] = mid;
                    num_pieces -= num_pieces_in_first_half;
                    piece -= num_pieces_in_first_half;
                }
            }
        }

        1
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkExtentTranslator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkExtentTranslator" || Object::is_type_of(name)
    }

    /// VTK: `vtkExtentTranslator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for ExtentTranslator {
    fn default() -> Self {
        Self::new()
    }
}

fn choose_split_axis(size: [i32; 3], split_mode: i32) -> i32 {
    if (0..3).contains(&split_mode) && size[split_mode as usize] > 1 {
        split_mode
    } else if size[2] >= size[1] && size[2] >= size[0] && size[2] / 2 >= 1 {
        2
    } else if size[1] >= size[0] && size[1] / 2 >= 1 {
        1
    } else if size[0] / 2 >= 1 {
        0
    } else {
        -1
    }
}
