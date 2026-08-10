use std::ffi::CStr;
use std::mem::MaybeUninit;

use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkSocket`.
#[derive(Debug)]
pub struct Socket {
    object: Object,
    socket_descriptor: i32,
    bound_address: String,
}

impl Socket {
    /// VTK: `vtkSocket::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkSocket"),
            socket_descriptor: -1,
            bound_address: String::new(),
        }
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            socket_descriptor: -1,
            bound_address: String::new(),
        }
    }

    /// VTK: `vtkSocket::GetConnected`.
    pub fn get_connected(&self) -> i32 {
        i32::from(self.socket_descriptor >= 0)
    }

    /// VTK: `vtkSocket::CloseSocket`.
    pub fn close_socket(&mut self) {
        self.close_socket_descriptor(self.socket_descriptor);
        self.socket_descriptor = -1;
    }

    /// VTK: `vtkSocket::Send`.
    pub fn send(&self, data: &[u8]) -> i32 {
        if self.get_connected() == 0 {
            return 0;
        }
        if data.is_empty() {
            return 1;
        }

        let mut total = 0usize;
        while total < data.len() {
            let ret = restart_interrupted_ssize(|| unsafe {
                libc::send(
                    self.socket_descriptor,
                    data[total..].as_ptr().cast(),
                    data.len() - total,
                    0,
                )
            });
            if ret == -1 {
                return 0;
            }
            total += ret as usize;
        }
        1
    }

    /// VTK: `vtkSocket::Receive`.
    pub fn receive(&self, data: &mut [u8], read_fully: i32) -> i32 {
        if self.get_connected() == 0 {
            return 0;
        }

        let mut total = 0usize;
        loop {
            let ret = restart_interrupted_ssize(|| unsafe {
                libc::recv(
                    self.socket_descriptor,
                    data[total..].as_mut_ptr().cast(),
                    data.len() - total,
                    0,
                )
            });
            if ret <= 0 {
                return 0;
            }
            total += ret as usize;
            if read_fully == 0 || total >= data.len() {
                return total as i32;
            }
        }
    }

    /// VTK: `vtkSocket::GetSocketDescriptor`.
    pub fn get_socket_descriptor(&self) -> i32 {
        self.socket_descriptor
    }

    /// VTK: `vtkSocket::GetBoundAddress`.
    pub fn get_bound_address(&self) -> &str {
        &self.bound_address
    }

    /// VTK: `vtkSocket::SelectSockets`.
    pub fn select_sockets(sockets_to_select: &[i32], msec: u64, selected_index: &mut i32) -> i32 {
        *selected_index = -1;
        if sockets_to_select.is_empty() {
            return 0;
        }

        let mut rset = MaybeUninit::<libc::fd_set>::uninit();
        let mut max_fd = -1;
        let ret = restart_interrupted(|| unsafe {
            libc::FD_ZERO(rset.as_mut_ptr());
            let rset_ptr = rset.as_mut_ptr();
            for &fd in sockets_to_select {
                libc::FD_SET(fd, rset_ptr);
                max_fd = max_fd.max(fd);
            }
            let mut timeout = timeval_from_msec(msec);
            let timeout_ptr = if msec > 0 {
                &mut timeout as *mut libc::timeval
            } else {
                std::ptr::null_mut()
            };
            libc::select(
                max_fd + 1,
                rset_ptr,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                timeout_ptr,
            )
        });

        if ret <= 0 {
            return ret as i32;
        }

        let rset = unsafe { rset.assume_init() };
        for (index, &fd) in sockets_to_select.iter().enumerate() {
            if unsafe { libc::FD_ISSET(fd, &rset) } {
                *selected_index = index as i32;
                return 1;
            }
        }
        -1
    }

    pub(crate) fn set_socket_descriptor(&mut self, socket_descriptor: i32) {
        self.socket_descriptor = socket_descriptor;
    }

    pub(crate) fn create_socket(&mut self) -> i32 {
        self.bound_address.clear();
        let sock =
            restart_interrupted(|| unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) });
        if sock == -1 {
            return -1;
        }

        let on: libc::c_int = 1;
        let ret = restart_interrupted(|| unsafe {
            libc::setsockopt(
                sock,
                libc::IPPROTO_TCP,
                libc::TCP_NODELAY,
                (&on as *const libc::c_int).cast(),
                std::mem::size_of_val(&on) as libc::socklen_t,
            )
        });
        if ret == -1 {
            close_descriptor(sock);
            return -1;
        }
        sock
    }

    pub(crate) fn close_socket_descriptor(&self, socket_descriptor: i32) {
        if socket_descriptor >= 0 {
            close_descriptor(socket_descriptor);
        }
    }

    pub(crate) fn bind_socket(
        &mut self,
        socket_descriptor: i32,
        port: i32,
        bind_addr: &str,
    ) -> i32 {
        let Ok(addr) = parse_ipv4_le(bind_addr) else {
            return -1;
        };

        let server = libc::sockaddr_in {
            sin_family: libc::AF_INET as libc::sa_family_t,
            sin_port: (port as u16).to_be(),
            sin_addr: libc::in_addr { s_addr: addr },
            sin_zero: [0; 8],
        };

        let opt: libc::c_int = 1;
        let _ = restart_interrupted(|| unsafe {
            libc::setsockopt(
                socket_descriptor,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                (&opt as *const libc::c_int).cast(),
                std::mem::size_of_val(&opt) as libc::socklen_t,
            )
        });

        let ret = restart_interrupted(|| unsafe {
            libc::bind(
                socket_descriptor,
                (&server as *const libc::sockaddr_in).cast(),
                std::mem::size_of_val(&server) as libc::socklen_t,
            )
        });
        if ret == -1 {
            return -1;
        }

        self.bound_address = bind_addr.to_string();
        0
    }

    pub(crate) fn select_socket(&self, socket_descriptor: i32, msec: u64) -> i32 {
        if socket_descriptor < 0 {
            return -1;
        }
        let mut selected_index = -1;
        Self::select_sockets(&[socket_descriptor], msec, &mut selected_index)
    }

    pub(crate) fn accept(&self, socket_descriptor: i32) -> i32 {
        if socket_descriptor < 0 {
            return -1;
        }
        restart_interrupted(|| unsafe {
            libc::accept(
                socket_descriptor,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        }) as i32
    }

    pub(crate) fn listen(&self, socket_descriptor: i32) -> i32 {
        if socket_descriptor < 0 {
            return -1;
        }
        restart_interrupted(|| unsafe { libc::listen(socket_descriptor, 1) }) as i32
    }

    pub(crate) fn connect(&self, socket_descriptor: i32, host_name: &str, port: i32) -> i32 {
        if socket_descriptor < 0 {
            return -1;
        }
        let Some(name) = resolve_ipv4(host_name, port) else {
            return -1;
        };

        let ret = unsafe {
            libc::connect(
                socket_descriptor,
                (&name as *const libc::sockaddr_in).cast(),
                std::mem::size_of_val(&name) as libc::socklen_t,
            )
        };
        if ret == -1 && errno() == libc::EINTR {
            return self.select_socket(socket_descriptor, 0);
        }
        ret as i32
    }

    pub(crate) fn get_port(&self, socket_descriptor: i32) -> i32 {
        let mut sockinfo = libc::sockaddr_in {
            sin_family: 0,
            sin_port: 0,
            sin_addr: libc::in_addr { s_addr: 0 },
            sin_zero: [0; 8],
        };
        let mut sizebuf = std::mem::size_of_val(&sockinfo) as libc::socklen_t;
        let ret = restart_interrupted(|| unsafe {
            libc::getsockname(
                socket_descriptor,
                (&mut sockinfo as *mut libc::sockaddr_in).cast(),
                &mut sizebuf,
            )
        });
        if ret == -1 {
            0
        } else {
            u16::from_be(sockinfo.sin_port) as i32
        }
    }

    /// VTK: `vtkSocket::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "SocketDescriptor: {}\nBoundAddress: {}\n",
            self.socket_descriptor, self.bound_address
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

impl Drop for Socket {
    fn drop(&mut self) {
        if self.socket_descriptor != -1 {
            self.close_socket_descriptor(self.socket_descriptor);
            self.socket_descriptor = -1;
        }
    }
}

impl Default for Socket {
    fn default() -> Self {
        Self::new()
    }
}

fn restart_interrupted(mut call: impl FnMut() -> libc::c_int) -> libc::c_int {
    loop {
        let ret = call();
        if ret != -1 || errno() != libc::EINTR {
            return ret;
        }
    }
}

fn restart_interrupted_ssize(mut call: impl FnMut() -> libc::ssize_t) -> libc::ssize_t {
    loop {
        let ret = call();
        if ret != -1 || errno() != libc::EINTR {
            return ret;
        }
    }
}

fn close_descriptor(socket_descriptor: i32) {
    let _ = restart_interrupted(|| unsafe { libc::close(socket_descriptor) });
}

fn resolve_ipv4(host_name: &str, port: i32) -> Option<libc::sockaddr_in> {
    let host_name = std::ffi::CString::new(host_name).ok()?;
    let service = std::ffi::CString::new(port.to_string()).ok()?;
    let hints = libc::addrinfo {
        ai_flags: 0,
        ai_family: libc::AF_INET,
        ai_socktype: libc::SOCK_STREAM,
        ai_protocol: 0,
        ai_addrlen: 0,
        ai_addr: std::ptr::null_mut(),
        ai_canonname: std::ptr::null_mut(),
        ai_next: std::ptr::null_mut(),
    };
    let mut result = std::ptr::null_mut();
    let status =
        unsafe { libc::getaddrinfo(host_name.as_ptr(), service.as_ptr(), &hints, &mut result) };
    if status != 0 || result.is_null() {
        return None;
    }

    let mut current = result;
    let mut resolved = None;
    while !current.is_null() {
        let info = unsafe { &*current };
        if info.ai_family == libc::AF_INET && !info.ai_addr.is_null() {
            resolved = Some(unsafe { *(info.ai_addr as *const libc::sockaddr_in) });
            break;
        }
        current = info.ai_next;
    }

    unsafe {
        libc::freeaddrinfo(result);
    }
    resolved
}

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

fn timeval_from_msec(msec: u64) -> libc::timeval {
    libc::timeval {
        tv_sec: (msec / 1000) as libc::time_t,
        tv_usec: ((msec % 1000) * 1000) as libc::suseconds_t,
    }
}

fn parse_ipv4_le(bind_addr: &str) -> Result<u32, ()> {
    let mut addr = 0u32;
    let mut count = 0usize;
    for (section, part) in bind_addr.split('.').enumerate() {
        let byte: u32 = part.parse().map_err(|_| ())?;
        if byte > 255 {
            return Err(());
        }
        addr += byte << (8 * section);
        count += 1;
    }
    if count == 4 {
        Ok(addr)
    } else {
        Err(())
    }
}

#[allow(dead_code)]
fn socket_error_text() -> Option<String> {
    let ptr = unsafe { libc::strerror(errno()) };
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}
