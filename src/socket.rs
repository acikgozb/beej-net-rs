use core::fmt;
use std::{
    error,
    ffi::{CStr, CString},
    io, mem, ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Showcases how `socket()` can be used.
// Section 5.2 - `socket()` - Get the File Descriptor!
// MANPAGE: man 3 socket
pub fn socket() -> Result<(), Error> {
    // Preparing the getaddrinfo call.
    let node = CString::new("www.example.com").unwrap();
    let node_ptr = node.as_ptr();

    let service = CString::new("http").unwrap();
    let service_ptr = service.as_ptr();

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY: all the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    let sock_fd = unsafe {
        let s = libc::getaddrinfo(node_ptr, service_ptr, &hints, &mut res_ptr);
        if s != 0 {
            let err = CStr::from_ptr(libc::gai_strerror(s)).to_string_lossy();
            return Err(Error::Getaddrinfo(err.into_owned()));
        }

        let res = *res_ptr;

        let sock_fd = libc::socket(res.ai_family, res.ai_socktype, 0);
        if sock_fd == -1 {
            let err = io::Error::last_os_error();
            return Err(Error::Socket(err));
        }

        libc::freeaddrinfo(res_ptr);
        sock_fd
    };

    println!("created sock fd: {}", sock_fd);

    Ok(())
}
