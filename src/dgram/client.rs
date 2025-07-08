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
    Close(io::Error),
    Sendto(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Close(err) => write!(f, "close error: {}", err),
            Error::Sendto(err) => write!(f, "sendto error: {}", err),
        }
    }
}

impl error::Error for Error {}

pub fn client() -> Result<(), Error> {
    let node = ptr::null();
    let port = CString::from(c"4950");

    // SAFETY: All zero hints is a valid initialization.
    // Required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_INET6;
    hints.ai_socktype = libc::SOCK_DGRAM;

    let mut gai_res_ptr: *mut libc::addrinfo = ptr::null_mut();

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
        let gai_res = unsafe { *gai_res_ptr };
        let next_res_ptr = gai_res.ai_next;

        // SAFETY: `socket()` is safe to call since `gai_res` is valid.
        let sock = unsafe { libc::socket(gai_res.ai_family, gai_res.ai_socktype, 0) };
        if sock == -1 {
            if next_res_ptr.is_null() {
                return Err(Error::Socket(io::Error::last_os_error()));
            } else {
                gai_res_ptr = next_res_ptr;
                continue;
            }
        }

        sock_fd = sock;
        break;
    }

    let msg_buf = b"Hello UDP server!";
    let len = msg_buf.len();

    // SAFETY: All `sendto()` arguments are initialized as desired.
    // There are no reads to uninitialized memory, therefore it is safe to call.
    let bytes = unsafe {
        let gai_res = { *gai_res_ptr };

        libc::sendto(
            sock_fd,
            msg_buf.as_ptr() as *const libc::c_void,
            len,
            0,
            gai_res.ai_addr,
            gai_res.ai_addrlen,
        )
    };
    match bytes {
        v if v > 0 => Ok(()),
        _ => Err(Error::Sendto(io::Error::last_os_error())),
    }?;

    // SAFETY: `gai_res` is no longer needed and its pointer points to a valid `addrinfo` struct at this point. It can be freed safely.
    unsafe {
        libc::freeaddrinfo(gai_res_ptr);
    }

    println!("talker: sent {} bytes", bytes);

    // SAFETY: `sock_fd` is not needed from now on.
    // It is safe to call `close()`.
    let ecode = unsafe { libc::close(sock_fd) };
    match ecode {
        -1 => Err(Error::Close(io::Error::last_os_error())),
        _ => Ok(()),
    }
}
