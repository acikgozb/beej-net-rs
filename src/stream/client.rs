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
    Connect(io::Error),
    Recv(io::Error),
    Close(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Connect(err) => write!(f, "connect error: {}", err),
            Error::Recv(err) => write!(f, "recv error: {}", err),
            Error::Close(err) => write!(f, "close err: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: A simple stream client that connects to the server created by `bjrs stream server` command.
// This example is a more complete version of `recv()` syscall example.
// MANPAGE:
// man 2 recv (Linux)
// man 3 recv (POSIX)
// man errno
pub fn client() -> Result<(), Error> {
    let node = ptr::null();
    let port = CString::from(c"3490");

    // SAFETY: All zero hints is a valid initialization.
    // Required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut gai_res_ptr = ptr::null_mut();

    // SAFETY: There is no uninitialized memory access. `getaddrinfo()` is safe to call.
    let ecode = unsafe { libc::getaddrinfo(node, port.as_ptr(), &hints, &mut gai_res_ptr) };
    match ecode {
        0 => Ok(()),
        _ => {
            // SAFETY: `gai_strerror` is valid to call on a failed `getaddrinfo()` call.
            let err = unsafe { CStr::from_ptr(libc::gai_strerror(ecode)).to_string_lossy() };
            Err(Error::Getaddrinfo(err.into_owned()))
        }
    }?;

    let mut sock_fd = -1;
    while !gai_res_ptr.is_null() {
        // SAFETY: `gai_res_ptr` is guaranteed to point atleast one valid addrinfo struct on a successful `getaddrinfo()` call.
        let gai_res = unsafe { *gai_res_ptr };
        let next_res_ptr = gai_res.ai_next;

        // SAFETY: `socket()` is safe to call since `gai_res` is valid.
        let sock = unsafe {
            let sock = libc::socket(gai_res.ai_family, gai_res.ai_socktype, 0);
            if sock == -1 {
                if next_res_ptr.is_null() {
                    return Err(Error::Socket(io::Error::last_os_error()));
                } else {
                    gai_res_ptr = next_res_ptr;
                    continue;
                }
            }

            sock
        };

        // SAFETY: `connect()` is safe to call since `sock` and `gai_res` are valid..
        let ecode = unsafe { libc::connect(sock, gai_res.ai_addr, gai_res.ai_addrlen) };
        if ecode == -1 {
            if next_res_ptr.is_null() {
                return Err(Error::Connect(io::Error::last_os_error()));
            } else {
                gai_res_ptr = next_res_ptr;
                continue;
            }
        }

        sock_fd = sock;
        break;
    }

    // SAFETY: `gai_res` is no longer needed and its pointer points to a valid `addrinfo` struct at this point. It can be freed safely.
    unsafe {
        libc::freeaddrinfo(gai_res_ptr);
    }

    const MAXDATASIZE: usize = 100;
    let mut recv_buf = vec![0; MAXDATASIZE];
    let len = recv_buf.len();

    // SAFETY:
    // 1 - `sock_fd` is a valid sock fd for server communication.
    // 2 - `recv_buf` and its len are initialized as desired.
    let bytes = unsafe { libc::recv(sock_fd, recv_buf.as_mut_ptr() as *mut libc::c_void, len, 0) };
    match bytes {
        -1 => Err(Error::Recv(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    recv_buf[bytes as usize] = b'\0';

    let msg = [b"client: received ", &recv_buf[..]].concat();
    io::stdout()
        .write_all(&msg)
        .expect("message to be written to stdout");

    // SAFETY:
    // `sock_fd` is a valid sock fd for peer communication.
    let ecode = unsafe { libc::close(sock_fd) };
    match ecode {
        -1 => Err(Error::Close(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    Ok(())
}
