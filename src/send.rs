use std::{error, fmt, io};

use crate::accept;

#[derive(Debug)]
pub enum Error {
    Accept(accept::Error),
    Send(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Accept(err) => {
                write!(f, "failed to get accepted connection sock fd: {}", err)
            }
            Error::Send(err) => write!(f, "send err: {}", err),
        }
    }
}

impl error::Error for Error {}

impl From<accept::Error> for Error {
    fn from(value: accept::Error) -> Self {
        Self::Accept(value)
    }
}

// EXAMPLE: Send an arbitrary data "hello world!" to socket created for an accepted connection to localhost, to port 3490.
// MANPAGE:
// man 2 send (Linux)
// man 3 send (POSIX)
pub fn send() -> Result<(), Error> {
    // NOTE: Since the example about `send()` is a pseudo-code, it is decided to use `accept()` to set up the process beforehand.
    let conn_sock_fd = crate::accept()?;

    let buf = b"hello world!\n";
    let len = buf.len();

    // SAFETY: For example purposes, the `send()` call is explicitly not checked to see whether all of buf is sent through the sock or not.
    // `send()` is just checked to see whether it succeeded or not.
    // Since the `conn_sock_fd` contains a initialized socket, and a fixed buf is used, it is safe to use `send()`.
    unsafe {
        let bytes_sent = libc::send(conn_sock_fd, buf.as_ptr() as *const libc::c_void, len, 0);
        match bytes_sent {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Send(err))
            }
            _ => Ok(()),
        }
    }?;

    Ok(())
}
