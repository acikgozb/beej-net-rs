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

    let mut res_ptr = ptr::null_mut();

    // SAFETY:
    // All the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    //
    // Having a one big unsafe block is just for showcase purposes.
    unsafe {
        let s = libc::getaddrinfo(node_ptr, service_ptr, &hints, &mut res_ptr);
        if s != 0 {
            let err = libc::gai_strerror(s);
            let c_err = CStr::from_ptr(err).to_string_lossy();
            return Err(Error::Getaddrinfo(c_err.into_owned()));
        }

        // SAFETY: `res_ptr` is initialized upon a successful getaddrinfo() call.
        // Therefore we can guarantee that there is atleast one addrinfo that `res_ptr` points to, making deref safe in the usages below.
        let res = *res_ptr;

        let sock_fd = libc::socket(res.ai_family, res.ai_socktype, 0);
        if sock_fd == -1 {
            let err = io::Error::last_os_error();
            return Err(Error::Socket(err));
        }

        // SAFETY: `bind()` is called on a valid `sock_fd` upon a successful `socket()` call.
        let s = libc::bind(sock_fd, res.ai_addr, res.ai_addrlen);
        if s != 0 {
            let err = io::Error::last_os_error();
            return Err(Error::Bind(sock_fd, err));
        }

        // SAFETY: `res_ptr` will not be used after this call, therefore it is safe to free it.
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

    let mut res_ptr = ptr::null_mut();

    // SAFETY:
    // All the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    //
    // Having a one big unsafe block is just for showcase purposes.
    unsafe {
        let s = libc::getaddrinfo(node_ptr, service_ptr, &hints, &mut res_ptr);
        if s != 0 {
            let err = CStr::from_ptr(libc::gai_strerror(s)).to_string_lossy();
            return Err(Error::Getaddrinfo(err.into_owned()));
        }

        // SAFETY: `res_ptr` is initialized upon a successful `getaddrinfo()` call.
        // Therefore we can guarantee that there is atleast one addrinfo that `res_ptr` points to, making deref safe in the usages below.
        let res = *res_ptr;

        let sock_fd = libc::socket(res.ai_family, res.ai_socktype, 0);
        if sock_fd == -1 {
            let err = io::Error::last_os_error();
            return Err(Error::Socket(err));
        }

        let reuse_addr = 1;

        // SAFETY: `setsockopt()` is called for a valid sock_fd created by a successful `socket()` call.
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

        // SAFETY: `bind()` is called on a valid `sock_fd` upon a successful `socket()` call.
        let s = libc::bind(sock_fd, res.ai_addr, res.ai_addrlen);
        if s != 0 {
            let err = io::Error::last_os_error();
            return Err(Error::Bind(sock_fd, err));
        }

        // SAFETY: `res_ptr` will not be used after this call, therefore it is safe to free it.
        libc::freeaddrinfo(res_ptr);
    }

    Ok(())
}
