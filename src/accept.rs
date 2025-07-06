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
    Listen(i32, io::Error),
    Accept(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Bind(sock_fd, err) => write!(f, "bind error on sock fd {}: {}", sock_fd, err),
            Error::Listen(sock_fd, err) => {
                write!(f, "listen error on sock fd {}: {}", sock_fd, err)
            }
            Error::Accept(err) => write!(f, "accept error: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Accepting an incoming connection that comes to localhost, to port 3490.
// MANPAGES:
// man 2 accept (Linux)
// man 3 accept (POSIX)
pub fn accept() -> Result<i32, Error> {
    let node = ptr::null() as *const libc::c_char;
    let port = CString::from(c"3490");

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY:
    // All the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    unsafe {
        let s = libc::getaddrinfo(node, port.as_ptr(), &hints, &mut res_ptr);
        match s {
            0 => Ok(()),
            _ => {
                let err = CStr::from_ptr(libc::gai_strerror(s)).to_string_lossy();
                Err(Error::Getaddrinfo(err.into_owned()))
            }
        }
    }?;

    // SAFETY: Since we are trying to get our loopback IP address via `getaddrinfo()`, we know that `res_ptr` points to an initialized memory, making `socket()` safe to use.
    // Any potential `socket()` error is checked by reading `errno` instantly after the `socket()` call. This ensures that `sock_fd` contains the fd of a successfully created socket.
    let sock_fd = unsafe {
        let res = *res_ptr;

        let fd = libc::socket(res.ai_family, res.ai_socktype, 0);
        match fd {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Socket(err))
            }
            fd => Ok(fd),
        }
    }?;

    // SAFETY: Due to the points above, `res_ptr` and `sock_fd` are safe to use.
    // Any potential `bind()` error is checked by reading `errno` instantly after the `bind()` call.
    // This ensures that any errors that may happen in `bind()` are caught.
    //
    // Since `res_ptr` points to a valid initialized memory and will not be used after `bind()`, it is safe to free it upon a successful `bind()` call.
    unsafe {
        let res = *res_ptr;
        let s = libc::bind(sock_fd, res.ai_addr, res.ai_addrlen);
        let res = match s {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Bind(sock_fd, err))
            }
            _ => Ok(()),
        };

        libc::freeaddrinfo(res_ptr);

        res
    }?;

    // SAFETY: The `sock_fd` used for `listen()` is guaranteed to be valid due to the points above.
    // Any potential `listen()` error is checked by reading `errno` instantly after the `listen()` call.
    unsafe {
        const BACKLOG: i32 = 10;

        let s = libc::listen(sock_fd, BACKLOG);
        match s {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Listen(sock_fd, err))
            }
            _ => Ok(()),
        }
    }?;

    println!("listening on port {}", port.to_string_lossy());

    // SAFETY: The uninitialized memory of `*addr_ptr` is initialized via `accept()`. This memory will hold the object regarding the accepted connection.
    // Any potential `accept()` error is checked by reading `errno` instantly after the `accept()` call.
    // The returned sock_fd is a valid fd created by a successful `accept()` call to interact with the accepted connection.
    let conn_sock_fd = unsafe {
        let addr_ptr: *mut libc::sockaddr_storage = ptr::null_mut();
        let addr_size = mem::size_of::<libc::sockaddr_storage>();

        let conn_sock_fd = libc::accept(
            sock_fd,
            addr_ptr as *mut libc::sockaddr,
            addr_size as *mut u32,
        );
        match conn_sock_fd {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Accept(err))
            }
            _ => Ok(conn_sock_fd),
        }
    }?;

    println!("sock fd of accepted connection: {}", conn_sock_fd);

    Ok(conn_sock_fd)
}
