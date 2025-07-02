use std::{
    error,
    ffi::{CStr, CString},
    fmt, io, mem, ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Bind(i32, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Bind(sock_fd, err) => write!(f, "bind error for sock_fd {}: {}", sock_fd, err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Bind a socket to the localhost, to the port 3490.
// Section 5.3 - `bind()` - What Port Am I On?
// MANPAGE: man 3 bind
pub fn bind() -> Result<(), Error> {
    // Preparing the getaddrinfo call.
    let node_ptr = ptr::null();

    let service = CString::new("3490").unwrap();
    let service_ptr = service.as_ptr();

    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_PASSIVE;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    unsafe {
        let s = libc::getaddrinfo(node_ptr, service_ptr, &hints, &mut res_ptr);
        if s != 0 {
            let err = libc::gai_strerror(s);
            let c_err = CStr::from_ptr(err).to_string_lossy();
            return Err(Error::Getaddrinfo(c_err.into_owned()));
        }

        let res = *res_ptr;

        let sock_fd = libc::socket(res.ai_family, res.ai_socktype, 0);
        if sock_fd == -1 {
            let err = io::Error::last_os_error();
            return Err(Error::Socket(err));
        }

        let s = libc::bind(sock_fd, res.ai_addr, res.ai_addrlen);
        if s != 0 {
            let err = io::Error::last_os_error();
            return Err(Error::Bind(sock_fd, err));
        }

        libc::freeaddrinfo(res_ptr);
    }

    Ok(())
}
