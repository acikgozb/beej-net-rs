use std::{
    error,
    ffi::{CStr, CString},
    fmt, io, mem, ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Connect(i32, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(error) => write!(f, "getaddrinfo error: {}", error),
            Error::Socket(error) => write!(f, "socket error: {}", error),
            Error::Connect(sock_fd, error) => {
                write!(f, "connect error on sock fd {}: {}", sock_fd, error)
            }
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Making a socket connection to www.example.com, port 3490.
// MANPAGE:
// man 2 connect (Linux)
// man 3 connect (POSIX)
pub fn connect() -> Result<(), Error> {
    // At this point, getaddrinfo is basically our bread and butter.
    let node = CString::from(c"www.example.com");
    let port = CString::from(c"3490");

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY:
    // 1 - all the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    // The memory used by getaddrinfo() is cleaned up at the end.
    // 2 - It is guaranteed to get atleast one address from getaddrinfo(),
    // because the address we use is publicly available on the Internet. This makes socket() safe to use.
    // 3 - The sock fd used in connect() is properly configured by using the single source of truth, getaddrinfo().
    // 4 - The errors are checked based on the return values of getaddrinfo(), socket(), and connect().
    //
    // Having a one big unsafe block is just for showcase purposes.
    unsafe {
        let s = libc::getaddrinfo(node.as_ptr(), port.as_ptr(), &hints, &mut res_ptr);
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

        let s = libc::connect(sock_fd, res.ai_addr, res.ai_addrlen);
        if s == -1 {
            let err = io::Error::last_os_error();
            return Err(Error::Connect(sock_fd, err));
        }

        libc::freeaddrinfo(res_ptr);
    }

    Ok(())
}
