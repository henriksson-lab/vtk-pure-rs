use crate::common::core::VtkMTimeType;

use super::socket::Socket;

/// VTK: `vtkClientSocket`.
#[derive(Debug)]
pub struct ClientSocket {
    socket: Socket,
    connecting_side: bool,
}

impl ClientSocket {
    /// VTK: `vtkClientSocket::New`.
    pub fn new() -> Self {
        Self {
            socket: Socket::with_class_name("vtkClientSocket"),
            connecting_side: false,
        }
    }

    /// VTK: `vtkClientSocket::ConnectToServer`.
    pub fn connect_to_server(&mut self, host_name: &str, port: i32) -> i32 {
        if self.socket.get_socket_descriptor() != -1 {
            self.socket
                .close_socket_descriptor(self.socket.get_socket_descriptor());
            self.socket.set_socket_descriptor(-1);
        }

        let socket_descriptor = self.socket.create_socket();
        if socket_descriptor == -1 {
            return -1;
        }
        self.socket.set_socket_descriptor(socket_descriptor);

        if self.socket.connect(socket_descriptor, host_name, port) == -1 {
            self.socket.close_socket_descriptor(socket_descriptor);
            self.socket.set_socket_descriptor(-1);
            return -1;
        }

        self.connecting_side = true;
        0
    }

    /// VTK: `vtkClientSocket::GetConnectingSide`.
    pub fn get_connecting_side(&self) -> bool {
        self.connecting_side
    }

    pub(crate) fn set_connecting_side(&mut self, connecting_side: bool) {
        self.connecting_side = connecting_side;
    }

    pub(crate) fn set_socket_descriptor(&mut self, socket_descriptor: i32) {
        self.socket.set_socket_descriptor(socket_descriptor);
    }

    /// VTK: `vtkSocket::GetConnected`.
    pub fn get_connected(&self) -> i32 {
        self.socket.get_connected()
    }

    /// VTK: `vtkSocket::CloseSocket`.
    pub fn close_socket(&mut self) {
        self.socket.close_socket();
    }

    /// VTK: `vtkSocket::Send`.
    pub fn send(&self, data: &[u8]) -> i32 {
        self.socket.send(data)
    }

    /// VTK: `vtkSocket::Receive`.
    pub fn receive(&self, data: &mut [u8], read_fully: i32) -> i32 {
        self.socket.receive(data, read_fully)
    }

    /// VTK: `vtkSocket::GetSocketDescriptor`.
    pub fn get_socket_descriptor(&self) -> i32 {
        self.socket.get_socket_descriptor()
    }

    /// VTK: `vtkSocket::GetBoundAddress`.
    pub fn get_bound_address(&self) -> &str {
        self.socket.get_bound_address()
    }

    /// VTK: `vtkClientSocket::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}ConnectingSide: {}\n",
            self.socket.print_self(),
            self.connecting_side
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.socket.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.socket.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.socket.get_m_time()
    }
}

impl Default for ClientSocket {
    fn default() -> Self {
        Self::new()
    }
}
