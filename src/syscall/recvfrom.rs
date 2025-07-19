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
    Recvfrom(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "sock error: {}", err),
            Error::Bind(sock_fd, err) => write!(f, "bind error on sock fd {}: {}", sock_fd, err),
            Error::Recvfrom(err) => write!(f, "recvfrom error: {}", err),
        }
    }
}
impl error::Error for Error {}

// EXAMPLE: Receive a message that comes to a named SOCK_DGRAM socket on localhost (INET), on port 3490.
// MANPAGE:
// man 2 recvfrom (Linux)
// man 3 recvfrom (POSIX)
pub fn recvfrom() -> Result<(), Error> {
    let node_ptr = ptr::null();
    let port = CString::from(c"3490");

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_DGRAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY:
    // 1 - All the required vars are initialized for getaddrinfo().
    // 2 - gai_stderror() is used for error cases only.
    unsafe {
        let s = libc::getaddrinfo(node_ptr, port.as_ptr(), &hints, &mut res_ptr);
        match s {
            0 => Ok(()),
            s => {
                let err = CStr::from_ptr(libc::gai_strerror(s)).to_string_lossy();
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
    //
    // 2 - Any potential `bind()` error is checked by reading `errno` instantly after the `bind()` call.
    // This ensures that any errors that may happen in `bind()` are caught.
    //
    // 3 - Since `res_ptr` points to a valid initialized memory and will not be used after `bind()`, it is safe to free it upon a successful `bind()` call.
    unsafe {
        let res = *res_ptr;
        let s = libc::bind(sock_fd, res.ai_addr, res.ai_addrlen);
        let res = match s {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Bind(sock_fd, err))
            }
            _ => Ok(s),
        };

        libc::freeaddrinfo(res_ptr);

        res
    }?;

    let mut buf: Vec<u8> = vec![0; 30];
    let len = buf.len();

    // SAFETY:
    // 1 - `sock_fd` points to a valid socket.
    //
    // 2 - Since we do not control the incoming message, we may receive a lot more bytes than we expect. It is intentionally kept unchecked in here to focus on showing how a `recvfrom()` call is constructed.
    //
    // 3 - A big enough memory is allocated for `from_addr` by using `sockaddr_storage`.
    // Even though the source address is not used in the example, it is just added here to show the difference between `recv()` and `recvfrom()`.
    //
    // 4 - Any potential `recvfrom()` error is checked by reading `errno` instantly after the `recvfrom()` call.
    let recv_bytes = unsafe {
        let mut from_addr: libc::sockaddr_storage = mem::zeroed();
        let mut from_addr_len = mem::size_of_val(&from_addr) as u32;

        let bytes = libc::recvfrom(
            sock_fd,
            buf.as_mut_ptr() as _,
            len,
            0,
            &raw mut from_addr as _,
            &raw mut from_addr_len,
        );
        match bytes {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Recvfrom(err))
            }
            _ => Ok(bytes),
        }
    }?;

    let msg = [format!("received {} bytes: ", recv_bytes).as_bytes(), &buf].concat();
    io::stdout()
        .write_all(&msg)
        .expect("received msg to be written to stdout");

    Ok(())
}
