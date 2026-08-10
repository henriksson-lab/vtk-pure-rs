use std::{
    ffi::{c_char, CStr},
    ptr,
};

use crate::common::core::{Object, VtkMTimeType};

fn get_long_alignment() -> usize {
    #[repr(C)]
    struct TestAlignLong {
        pad: c_char,
        x: libc::c_long,
    }

    std::mem::offset_of!(TestAlignLong, x)
}

#[derive(Debug, Clone)]
struct HeapBlock {
    data: Box<[u8]>,
}

impl HeapBlock {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size].into_boxed_slice(),
        }
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
}

/// VTK: `vtkHeap`.
#[derive(Debug, Clone)]
pub struct Heap {
    object: Object,
    block_size: usize,
    number_of_allocations: i32,
    number_of_blocks: i32,
    alignment: usize,
    blocks: Vec<HeapBlock>,
    current: Option<usize>,
    position: usize,
}

impl Heap {
    /// VTK: `vtkHeap::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkHeap"),
            block_size: 256_000,
            number_of_allocations: 0,
            number_of_blocks: 0,
            alignment: get_long_alignment(),
            blocks: Vec::new(),
            current: None,
            position: 0,
        }
    }

    /// VTK: `vtkHeap::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Block Size: {}\nNumber of Blocks: {}\nNumber of Allocations: {}\nCurrent bytes allocated: {}\n",
            self.block_size,
            self.number_of_blocks,
            self.number_of_allocations,
            ((self.number_of_blocks - 1) as isize * self.block_size as isize
                + self.position as isize)
        )
    }

    /// VTK: `vtkHeap::AllocateMemory`.
    pub fn allocate_memory(&mut self, mut n: usize) -> *mut std::ffi::c_void {
        if n % self.alignment != 0 {
            n += self.alignment - (n % self.alignment);
        }

        let block_size = n.max(self.block_size);
        self.number_of_allocations += 1;

        if self.current.is_none()
            || self.position + n >= self.blocks[self.current.expect("current block")].size()
        {
            self.add(block_size);
        }

        let current = self.current.expect("current block");
        let ptr = unsafe { self.blocks[current].as_mut_ptr().add(self.position) };
        self.position += n;
        ptr.cast()
    }

    /// VTK: `vtkHeap::SetBlockSize`.
    pub fn set_block_size(&mut self, block_size: usize) {
        if self.block_size != block_size {
            self.block_size = block_size;
            self.modified();
        }
    }

    /// VTK: `vtkHeap::GetBlockSize`.
    pub fn get_block_size(&self) -> usize {
        self.block_size
    }

    /// VTK: `vtkHeap::GetNumberOfBlocks`.
    pub fn get_number_of_blocks(&self) -> i32 {
        self.number_of_blocks
    }

    /// VTK: `vtkHeap::GetNumberOfAllocations`.
    pub fn get_number_of_allocations(&self) -> i32 {
        self.number_of_allocations
    }

    /// VTK: `vtkHeap::Reset`.
    pub fn reset(&mut self) {
        self.current = if self.blocks.is_empty() {
            None
        } else {
            Some(0)
        };
        self.position = 0;
    }

    /// VTK: `vtkHeap::StringDup`.
    pub unsafe fn string_dup(&mut self, str: *const c_char) -> *mut c_char {
        let bytes = unsafe { CStr::from_ptr(str) }.to_bytes_with_nul();
        let new_str = self.allocate_memory(bytes.len()).cast::<c_char>();
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), new_str, bytes.len());
        }
        new_str
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkHeap::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkHeap" || Object::is_type_of(name)
    }

    /// VTK: `vtkHeap::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    fn add(&mut self, block_size: usize) {
        self.position = 0;

        if let Some(current) = self.current {
            let next = current + 1;
            if next < self.blocks.len() && self.blocks[next].size() >= block_size {
                self.current = Some(next);
                return;
            }
        }

        self.number_of_blocks += 1;
        self.blocks.push(HeapBlock::new(block_size));
        self.current = Some(self.blocks.len() - 1);
    }

    fn clean_all(&mut self) {
        self.current = if self.blocks.is_empty() {
            None
        } else {
            Some(0)
        };
        if self.current.is_none() {
            return;
        }
        while self.delete_and_next().is_some() {}
        self.current = None;
        self.position = 0;
    }

    fn delete_and_next(&mut self) -> Option<usize> {
        let current = self.current?;
        self.blocks.remove(current);
        if current < self.blocks.len() {
            self.current = Some(current);
        } else {
            self.current = None;
        }
        self.current
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        self.clean_all();
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}
