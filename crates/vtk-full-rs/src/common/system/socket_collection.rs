use std::ptr::{self, NonNull};

use crate::common::core::{Object, VtkMTimeType};

use super::socket::Socket;

/// VTK: `vtkSocketCollection`.
#[derive(Debug)]
pub struct SocketCollection {
    object: Object,
    current: usize,
    sockets: Vec<NonNull<Socket>>,
    selected_socket: Option<NonNull<Socket>>,
}

impl SocketCollection {
    /// VTK: `vtkSocketCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkSocketCollection"),
            current: 0,
            sockets: Vec::new(),
            selected_socket: None,
        }
    }

    /// VTK: `vtkSocketCollection::AddItem`.
    ///
    /// # Safety
    ///
    /// `socket` must point to a live `Socket` that remains valid while stored in
    /// this collection. This mirrors VTK's raw object-pointer collection shape.
    pub unsafe fn add_item(&mut self, socket: *mut Socket) {
        let socket = NonNull::new(socket).expect("vtkSocketCollection::AddItem socket is null");
        self.sockets.push(socket);
        self.modified();
    }

    /// VTK: `vtkSocketCollection::SelectSockets`.
    pub fn select_sockets(&mut self, msec: u64) -> i32 {
        self.selected_socket = None;

        if self.sockets.is_empty() {
            return -1;
        }

        let mut socket_indices = Vec::with_capacity(self.sockets.len());
        let mut sockets_to_select = Vec::with_capacity(self.sockets.len());

        for (index, socket) in self.sockets.iter().enumerate() {
            let socket = unsafe { socket.as_ref() };
            if socket.get_connected() == 0 {
                continue;
            }
            sockets_to_select.push(socket.get_socket_descriptor());
            socket_indices.push(index);
        }

        if sockets_to_select.is_empty() {
            return -1;
        }

        let mut selected_index = -1;
        let result = Socket::select_sockets(&sockets_to_select, msec, &mut selected_index);
        if result <= 0 || selected_index == -1 {
            return result;
        }

        let actual_index = socket_indices[selected_index as usize];
        self.selected_socket = self.sockets.get(actual_index).copied();
        1
    }

    /// VTK: `vtkSocketCollection::GetLastSelectedSocket`.
    pub fn get_last_selected_socket(&self) -> *mut Socket {
        self.selected_socket
            .map_or(ptr::null_mut(), |socket| socket.as_ptr())
    }

    /// VTK: `vtkSocketCollection::ReplaceItem`.
    ///
    /// # Safety
    ///
    /// `socket` must point to a live `Socket` that remains valid while stored in
    /// this collection.
    pub unsafe fn replace_item(&mut self, i: i32, socket: *mut Socket) {
        if i < 0 {
            return;
        }
        let index = i as usize;
        if index >= self.sockets.len() {
            return;
        }

        if self.selected_socket == Some(self.sockets[index]) {
            self.selected_socket = None;
        }
        self.sockets[index] =
            NonNull::new(socket).expect("vtkSocketCollection::ReplaceItem socket is null");
        self.modified();
    }

    /// VTK: `vtkSocketCollection::RemoveItem(int)`.
    pub fn remove_item_at(&mut self, i: i32) {
        if i < 0 {
            return;
        }
        let index = i as usize;
        if index >= self.sockets.len() {
            return;
        }

        if self.selected_socket == Some(self.sockets[index]) {
            self.selected_socket = None;
        }
        self.sockets.remove(index);
        if self.current > index {
            self.current -= 1;
        } else if self.current > self.sockets.len() {
            self.current = self.sockets.len();
        }
        self.modified();
    }

    /// VTK: `vtkSocketCollection::RemoveItem(vtkObject*)`.
    pub fn remove_item(&mut self, socket: *mut Socket) {
        let Some(socket) = NonNull::new(socket) else {
            return;
        };
        if let Some(index) = self.sockets.iter().position(|stored| *stored == socket) {
            self.remove_item_at(index as i32);
        }
    }

    /// VTK: `vtkSocketCollection::RemoveAllItems`.
    pub fn remove_all_items(&mut self) {
        if self.sockets.is_empty() && self.selected_socket.is_none() {
            return;
        }
        self.selected_socket = None;
        self.sockets.clear();
        self.current = 0;
        self.modified();
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.sockets.len() as i32
    }

    /// VTK: `vtkCollection::GetItemAsObject`.
    pub fn get_item_as_socket(&self, i: i32) -> *mut Socket {
        if i < 0 {
            return ptr::null_mut();
        }
        self.sockets
            .get(i as usize)
            .map_or(ptr::null_mut(), |socket| socket.as_ptr())
    }

    /// VTK: `vtkCollection::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.current = 0;
    }

    /// VTK: `vtkCollection::GetNextItemAsObject`.
    pub fn get_next_item_as_socket(&mut self) -> *mut Socket {
        if self.current >= self.sockets.len() {
            return ptr::null_mut();
        }
        let socket = self.sockets[self.current].as_ptr();
        self.current += 1;
        socket
    }

    /// VTK: `vtkSocketCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "NumberOfItems: {}\nSelectedSocket: {:p}\n",
            self.sockets.len(),
            self.get_last_selected_socket()
        )
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

impl Default for SocketCollection {
    fn default() -> Self {
        Self::new()
    }
}
