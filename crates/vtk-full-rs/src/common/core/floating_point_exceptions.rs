/// VTK: `vtkFloatingPointExceptions`.
pub struct FloatingPointExceptions;

impl FloatingPointExceptions {
    /// VTK: `vtkFloatingPointExceptions::Enable`.
    pub fn enable() {
        platform::enable();
    }

    /// VTK: `vtkFloatingPointExceptions::Disable`.
    pub fn disable() {
        platform::disable();
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod platform {
    const FE_INVALID: libc::c_int = 0x01;
    const FE_DIVBYZERO: libc::c_int = 0x04;
    const FPE_MASK: libc::c_int = FE_DIVBYZERO | FE_INVALID;

    pub(super) fn enable() {
        unsafe {
            feenableexcept(FPE_MASK);
            libc::signal(libc::SIGFPE, signal_handler as libc::sighandler_t);
        }
    }

    pub(super) fn disable() {
        unsafe {
            fedisableexcept(FPE_MASK);
        }
    }

    extern "C" fn signal_handler(signal: libc::c_int) {
        eprintln!("Error: Floating point exception detected. Signal {signal}");
        std::process::abort();
    }

    extern "C" {
        fn feenableexcept(excepts: libc::c_int) -> libc::c_int;
        fn fedisableexcept(excepts: libc::c_int) -> libc::c_int;
    }
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
mod platform {
    pub(super) fn enable() {}

    pub(super) fn disable() {}
}
