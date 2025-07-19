use std::{
    error,
    ffi::{CStr, CString},
    fmt, io, mem, ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Sendto(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo err: {}", err),
            Error::Socket(err) => write!(f, "sock err: {}", err),
            Error::Sendto(err) => write!(f, "sendto err: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Send a message via a SOCK_DGRAM socket to the UDP server on localhost (INET), on port 3490.
// MANPAGE:
// man 2 sendto (Linux)
// man 3 sendto (POSIX)
pub fn sendto() -> Result<(), Error> {
    // This time, we are working with a DGRAM socket.
    // Therefore, we are not using `accept()` like we did for `send()`.
    // We simply try to send a message through a SOCK_DGRAM configured for 127.0.0.1:3490.
    let node = ptr::null();
    let port = CString::from(c"3490");

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_DGRAM;

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
            _ => Ok(fd),
        }
    }?;

    let buf = b"hello world!\n";
    let len = buf.len();

    // SAFETY: Due to the points above, `*res_ptr` is safe to use.
    //
    // For example purposes, the `sendto()` call is explicitly not checked to see whether all of buf is sent through the sock or not.
    //
    // `sendto()` is just checked to see whether it succeeded or not.
    //
    // Since the `sock_fd` contains an initialized socket, and the buf is initialized, it is safe to use `sendto()`.
    unsafe {
        let res = *res_ptr;

        let bytes_sent = libc::sendto(
            sock_fd,
            buf.as_ptr() as *const libc::c_void,
            len,
            0,
            res.ai_addr,
            res.ai_addrlen,
        );
        match bytes_sent {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Sendto(err))
            }
            _ => Ok(()),
        }
    }?;

    // Since `res_ptr` points to a valid initialized memory and will not be used after `sendto()`, it is safe to free it upon a successful `sendto()` call.
    unsafe {
        libc::freeaddrinfo(res_ptr);
    }

    Ok(())
}
