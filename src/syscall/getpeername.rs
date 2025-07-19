use std::{
    error,
    ffi::{CStr, CString},
    fmt, io, mem,
    net::Ipv4Addr,
    ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Bind(io::Error),
    Listen(io::Error),
    Accept(io::Error),
    Getpeername(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Bind(err) => write!(f, "bind error: {}", err),
            Error::Listen(err) => write!(f, "listen error: {}", err),
            Error::Accept(err) => write!(f, "accept error: {}", err),
            Error::Getpeername(err) => write!(f, "getpeername error: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: See who is connected to the socket.
// MANPAGE:
// man 2 getpeername (Linux)
// man 2 getpeername (POSIX)
pub fn getpeername() -> Result<(), Error> {
    let node = ptr::null();
    let port = CString::from(c"3490");

    // SAFETY: hints is initialized as zeroes, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY:
    // 1 - All the required vars are initialized for getaddrinfo().
    // 2 - gai_stderror() is used for error cases only.
    unsafe {
        let ecode = libc::getaddrinfo(node, port.as_ptr(), &hints, &mut res_ptr);
        match ecode {
            0 => Ok(()),
            _ => {
                let err = CStr::from_ptr(libc::gai_strerror(ecode)).to_string_lossy();
                Err(Error::Getaddrinfo(err.into_owned()))
            }
        }
    }?;

    // SAFETY:
    // 1 - Since we are trying to get our loopback IP address via `getaddrinfo()`, we know that `res_ptr` points to an initialized memory, making `socket()` safe to use.
    // 2 - Any potential `socket()` error is checked by reading `errno` instantly after the `socket()` call. This ensures that `sock_fd` contains the fd of a successfully created socket.
    let sock_fd = unsafe {
        let res = *res_ptr;

        let fd = libc::socket(res.ai_family, res.ai_socktype, 0);
        match fd {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Socket(err))
            }
            _ => Ok(fd),
        }
    }?;

    // SAFETY:
    // 1 - Due to the points above, `res_ptr` and `sock_fd` are safe to use.
    // 2 - Any potential `bind()` error is checked by reading `errno` instantly after the `bind()` call.
    // This ensures that any errors that may happen in `bind()` are caught.
    unsafe {
        let res = *res_ptr;

        let ecode = libc::bind(sock_fd, res.ai_addr, res.ai_addrlen);
        match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Socket(err))
            }
            _ => Ok(()),
        }
    }?;

    // SAFETY: Since `res_ptr` points to a valid initialized memory and will not be used after `bind()`, it is safe to free it upon a successful `bind()` call.
    unsafe {
        libc::freeaddrinfo(res_ptr);
    }

    // SAFETY:
    // 1- The `sock_fd` used for `listen()` is guaranteed to be valid due to the points above.
    // 2 - Any potential `listen()` error is checked by reading `errno` instantly after the `listen()` call.
    unsafe {
        const BACKLOG: i32 = 10;

        let ecode = libc::listen(sock_fd, BACKLOG);
        match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Listen(err))
            }
            _ => Ok(()),
        }
    }?;

    // SAFETY:
    // 1 - Due to the points above, `sock_fd` is safe to use.
    // 2 - Any potential `accept()` error is checked by reading `errno` instantly after the `accept()` call.
    // 3 - The returned sock_fd is a valid fd created by a successful `accept()` call to interact with the accepted connection.
    let conn_sock_fd = unsafe {
        let fd = libc::accept(sock_fd, ptr::null_mut(), ptr::null_mut());
        match fd {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Accept(err))
            }
            _ => Ok(fd),
        }
    }?;

    // SAFETY:
    // 1 - Zeroed out `sockaddr_storage` is a valid initialization.
    // 2 - `conn_sock_fd` is a valid sock fd to use.
    // 3 - Any potential `accept()` error is checked by reading `errno` instantly after the `accept()` call.
    let sockaddr_storage = unsafe {
        let mut sockaddr_storage: libc::sockaddr_storage = mem::zeroed();
        let mut storage_len = mem::size_of_val(&sockaddr_storage);

        let ecode = libc::getpeername(
            conn_sock_fd,
            &raw mut sockaddr_storage as *mut libc::sockaddr,
            &raw mut storage_len as _,
        );
        match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Getpeername(err))
            }
            _ => Ok(sockaddr_storage),
        }
    }?;

    // SAFETY: `sockaddr_storage` is filled by a valid `getpeername()` call.
    // Therefore, reading from it is safe.
    let sockaddr_in = unsafe { *(&raw const sockaddr_storage as *const libc::sockaddr_in) };

    let bits = u32::from_be(sockaddr_in.sin_addr.s_addr);
    let from_addr = Ipv4Addr::from_bits(bits);
    println!(
        "peer ip addr: {}, port: {}",
        from_addr, sockaddr_in.sin_port
    );

    Ok(())
}
