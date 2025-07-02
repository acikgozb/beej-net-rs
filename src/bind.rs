use std::{
    error,
    ffi::{CStr, CString},
    fmt, io, mem, ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    SocketOpt(io::Error),
    Bind(i32, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {:?}", err),
            Error::SocketOpt(err) => write!(f, "setsockopt error: {}", err),
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

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_PASSIVE;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY:
    // 1 - all the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    // The memory used by getaddrinfo() is cleaned up at the end.
    // 2 - It is guaranteed to get atleast one address from getaddrinfo(),
    // due to using the loopback address and a port that does not need privileged access. This makes socket() safe to use.
    // 4 - For bind(), the created sock fd is used and due to getaddrinfo() returning a valid response, bind() reads valid memory.
    //
    // Having a one big unsafe block is just for showcase purposes.
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

// EXAMPLE: Allow a socket to reuse the port that was occupied
// by a socket before.
// Sometimes, a socket that was previously connected to the port may "hog" the port after it's disconnected.
// Section 5.3 - `bind()` - What Port Am I On?
// MANPAGE:
// - man 3 setsockopt
// - man 7 socket
pub fn reuse_port() -> Result<(), Error> {
    // Preparing the getaddrinfo call.
    let node_ptr = ptr::null();

    let service = CString::new("3490").unwrap();
    let service_ptr = service.as_ptr();

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_flags = libc::AI_PASSIVE;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY:
    // 1 - all the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    // The memory used by getaddrinfo() is cleaned up at the end.
    // 2 - It is guaranteed to get atleast one address from getaddrinfo(),
    // due to using the loopback address and a port that does not need privileged access. This makes socket() safe to use.
    // 3 - The memory accessed by setsockopt() is filled and valid to use.
    // 4 - For bind(), the created sock fd is used and due to getaddrinfo() returning a valid response, bind() reads valid memory.
    //
    // Having a one big unsafe block is just for showcase purposes.
    unsafe {
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

        let reuse_addr = 1;

        let s = libc::setsockopt(
            sock_fd,
            libc::SOL_SOCKET,
            libc::SO_REUSEADDR,
            &raw const reuse_addr as *const libc::c_void,
            mem::size_of::<i32>() as libc::socklen_t,
        );
        if s == -1 {
            let err = io::Error::last_os_error();
            return Err(Error::SocketOpt(err));
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
