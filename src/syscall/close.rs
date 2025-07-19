use std::{
    error,
    ffi::{CStr, CString},
    fmt, io, mem, ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Close(i32, io::Error),
    Send(i32, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket err: {}", err),
            Error::Close(sock_fd, err) => write!(
                f,
                "close err on sock fd {}: {}
                ",
                sock_fd, err
            ),
            Error::Send(sock_fd, err) => write!(f, "send err on sock fd {}: {}", sock_fd, err),
        }
    }
}
impl error::Error for Error {}

// EXAMPLE: Attempt to send a message on a closed socket via `close()`.
// MANPAGE:
// man 2 close (Linux)
// man 3 close (POSIX)
// man errno
pub fn close() -> Result<(), Error> {
    let node = ptr::null();
    let port = CString::from(c"3490");

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_DGRAM;

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
    // 1 - `sock_fd` points to a valid socket file descriptor created by `socket()`.
    // 2 - Any potential `close()` error is checked by reading `errno` instantly after the `close()` call.
    unsafe {
        let ecode = libc::close(sock_fd);
        match ecode {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Close(sock_fd, err))
            }
            _ => Ok(()),
        }
    }?;

    let buf = b"will this message be able to go through?";
    let len = buf.len();

    // SAFETY:
    // 1 - `sock_fd` points to a valid socket file descriptor created by `socket()`.
    // 2 - `res_ptr` points to a valid memory filled via `getaddrinfo()`.
    // 3 - The fixed message buf is initialized as a simple byte array.
    // 4 - Any potential `sendto()` error is checked by reading `errno` instantly after the `sendto()` call.
    // 5 - Since `res_ptr` is not used after `sendto()`, it can be freed without any side effects.
    let sent_bytes = unsafe {
        let res = *res_ptr;

        let bytes = libc::sendto(
            sock_fd,
            buf.as_ptr() as _,
            len,
            0,
            res.ai_addr,
            res.ai_addrlen,
        );

        let send_res = match bytes {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Send(sock_fd, err))
            }
            _ => Ok(bytes),
        };

        libc::freeaddrinfo(res_ptr);

        send_res
    }?;

    // We cannot reach the line below.
    // If you check the diagnostic message and compare it with errno values, you will see that `sendto()` fails with err `EBADF` err code.
    println!("sent {} bytes", sent_bytes);

    Ok(())
}
