use crate::common::core::VtkMTimeType;

use super::{client_socket::ClientSocket, socket::Socket};

/// VTK: `vtkServerSocket`.
#[derive(Debug)]
pub struct ServerSocket {
    socket: Socket,
}

impl ServerSocket {
    /// VTK: `vtkServerSocket::New`.
    pub fn new() -> Self {
        Self {
            socket: Socket::with_class_name("vtkServerSocket"),
        }
    }

    /// VTK: `vtkServerSocket::GetServerPort`.
    pub fn get_server_port(&self) -> i32 {
        if self.get_connected() == 0 {
            0
        } else {
            self.socket.get_port(self.socket.get_socket_descriptor())
        }
    }

    /// VTK: `vtkServerSocket::CreateServer`.
    pub fn create_server(&mut self, port: i32, bind_addr: &str) -> i32 {
        if self.socket.get_socket_descriptor() != -1 {
            self.socket
                .close_socket_descriptor(self.socket.get_socket_descriptor());
            self.socket.set_socket_descriptor(-1);
        }

        let socket_descriptor = self.socket.create_socket();
        if socket_descriptor < 0 {
            return -1;
        }
        self.socket.set_socket_descriptor(socket_descriptor);

        if self.socket.bind_socket(socket_descriptor, port, bind_addr) != 0
            || self.socket.listen(socket_descriptor) != 0
        {
            self.socket.close_socket_descriptor(socket_descriptor);
            self.socket.set_socket_descriptor(-1);
            return -1;
        }
        0
    }

    /// VTK: `vtkServerSocket::CreateServer(int port)`.
    pub fn create_server_on_any_address(&mut self, port: i32) -> i32 {
        self.create_server(port, "0.0.0.0")
    }

    /// VTK: `vtkServerSocket::WaitForConnection`.
    pub fn wait_for_connection(&self, msec: u64) -> Option<ClientSocket> {
        if self.socket.get_socket_descriptor() < 0 {
            return None;
        }

        match self
            .socket
            .select_socket(self.socket.get_socket_descriptor(), msec)
        {
            0 | -1 => None,
            _ => {
                let client_socket_descriptor =
                    self.socket.accept(self.socket.get_socket_descriptor());
                if client_socket_descriptor == -1 {
                    return None;
                }
                let mut client_socket = ClientSocket::new();
                client_socket.set_socket_descriptor(client_socket_descriptor);
                client_socket.set_connecting_side(false);
                Some(client_socket)
            }
        }
    }

    /// VTK: `vtkSocket::GetConnected`.
    pub fn get_connected(&self) -> i32 {
        self.socket.get_connected()
    }

    /// VTK: `vtkSocket::CloseSocket`.
    pub fn close_socket(&mut self) {
        self.socket.close_socket();
    }

    /// VTK: `vtkSocket::GetSocketDescriptor`.
    pub fn get_socket_descriptor(&self) -> i32 {
        self.socket.get_socket_descriptor()
    }

    /// VTK: `vtkSocket::GetBoundAddress`.
    pub fn get_bound_address(&self) -> &str {
        self.socket.get_bound_address()
    }

    /// VTK: `vtkServerSocket::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.socket.print_self()
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

impl Default for ServerSocket {
    fn default() -> Self {
        Self::new()
    }
}
