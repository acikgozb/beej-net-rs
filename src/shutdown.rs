use std::{
    error,
    ffi::{CStr, CString},
    fmt,
    io::{self, Write},
    mem, ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Bind(i32, io::Error),
    Listen(i32, io::Error),
    Accept(i32, io::Error),
    Shutdown(io::Error),
    Send(i32, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Bind(sock_fd, err) => write!(f, "bind error on sock fd {}: {}", sock_fd, err),
            Error::Listen(sock_fd, err) => {
                write!(f, "listen error on sock fd {}: {}", sock_fd, err)
            }
            Error::Accept(sock_fd, err) => {
                write!(f, "accept error on sock fd {}: {}", sock_fd, err)
            }
            Error::Shutdown(err) => write!(f, "shutdown error: {}", err),
            Error::Send(sock_fd, err) => {
                write!(f, "send error on sock fd {}: {}", sock_fd, err)
            }
        }
    }
}
impl error::Error for Error {}

// EXAMPLE: Showcase which operations are not allowed on a shutdowned socket.
// MANPAGE:
// man 2 shutdown (Linux)
// man 3 shutdown (POSIX)
// man 2 send (to see the reason of EPIPE error)
// man errno
pub fn shutdown() -> Result<(), Error> {
    let node = ptr::null();
    let port = CString::from(c"3490");

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
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
    // 3 - Since `res_ptr` points to a valid initialized memory and will not be used after `bind()`, it is safe to free it upon a successful `bind()` call.
    unsafe {
        let res = *res_ptr;

        let ecode = libc::bind(sock_fd, res.ai_addr, res.ai_addrlen);
        let res = match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Bind(sock_fd, err))
            }
            _ => Ok(()),
        };

        libc::freeaddrinfo(res_ptr);

        res
    }?;

    // SAFETY:
    // 1- The `sock_fd` used for `listen()` is guaranteed to be valid due to the points above.
    // 2 - Any potential `listen()` error is checked by reading `errno` instantly after the `listen()` call.
    unsafe {
        const BACKLOG: i32 = 10;

        let ecode = libc::listen(sock_fd, BACKLOG);
        match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Listen(sock_fd, err))
            }
            _ => Ok(()),
        }
    }?;

    // SAFETY:
    // 1- The uninitialized memory of `*addr_ptr` is initialized via `accept()`. This memory will hold the object regarding the accepted connection.
    // 2 - Any potential `accept()` error is checked by reading `errno` instantly after the `accept()` call.
    // 3 - The returned sock_fd is a valid fd created by a successful `accept()` call to interact with the accepted connection.
    let conn_sock_fd = unsafe {
        let from_addr: *mut libc::sockaddr_storage = ptr::null_mut();
        let from_addr_len = mem::size_of::<libc::sockaddr_storage>();

        let conn_sock_fd = libc::accept(
            sock_fd,
            from_addr as *mut libc::sockaddr,
            from_addr_len as *mut u32,
        );
        match conn_sock_fd {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Accept(sock_fd, err))
            }
            _ => Ok(conn_sock_fd),
        }
    }?;

    // SAFETY:
    // 1 - The `conn_sock_fd` is a valid socket fd initialized by a successful `accept()` call.
    // 2 - Any potential `shutdown()` error is checked by reading `errno` instantly after the `shutdown()` call.
    unsafe {
        let ecode = libc::shutdown(conn_sock_fd, 1);
        match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Shutdown(err))
            }
            _ => Ok(()),
        }
    }?;

    let send_buf = b"will this message be able to go through?";
    let len = send_buf.len();

    // SAFETY:
    // 1- For example purposes, the `send()` call is explicitly not checked to see whether all of buf is sent through the sock or not.
    // 2 - `send()` is just checked to see whether it succeeded or not.
    // 3 - Since the `conn_sock_fd` contains a initialized socket, and a fixed buf is used, it is safe to use `send()`.
    // 4 - Any potential `send()` error is checked by reading `errno` instantly after the `send()` call.
    unsafe {
        let ecode = libc::send(
            conn_sock_fd,
            send_buf.as_ptr() as *const libc::c_void,
            len,
            0,
        );
        match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Send(conn_sock_fd, err))
            }
            _ => Ok(()),
        }
    }?;

    let msg = [b"sent message: ", &send_buf[..]].concat();
    io::stdout()
        .write_all(&msg)
        .expect("message to be written to stdout");

    Ok(())
}
