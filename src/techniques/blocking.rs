use std::{error, fmt, io, ptr};

#[derive(Debug)]
pub enum Error {
    Socket(io::Error),
    Fcntl(io::Error),
    Recv(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Fcntl(err) => write!(f, "fcntl error: {}", err),
            Error::Recv(err) => write!(f, "recv error: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Attempt to recv from a non-blocking socket.
// MANPAGE:
// man 2 fcntl (Linux)
// man 3 fcntl (POSIX)
// man errno
pub fn blocking() -> Result<(), Error> {
    // SAFETY: There are no reads to uninitialized memory, making `socket()` safe to use.
    let sock = unsafe { libc::socket(libc::PF_INET, libc::SOCK_DGRAM, 0) };
    match sock {
        -1 => Err(Error::Socket(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    // SAFETY: `fnctl()` is called on a valid socket.
    let res = unsafe { libc::fcntl(sock, libc::F_SETFL, libc::O_NONBLOCK) };
    match res {
        -1 => Err(Error::Fcntl(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    // SAFETY: There are no reads to uninitialized memory, making `recvfrom()` safe to use.
    let bytes = unsafe {
        libc::recvfrom(
            sock,
            [0; 1].as_mut_ptr() as *mut libc::c_void,
            1,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    match bytes {
        // NOTE: EAGAIN or EWOULDBLOCK may be received from the OS.
        // Search the err message in `man errno` to find our the exact err code.
        -1 => Err(Error::Recv(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    // Bytes are intentionally printed here to observe that the process
    // cannot reach the line below.
    println!("received {} bytes", bytes);

    Ok(())
}
