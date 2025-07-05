use std::{
    error, fmt,
    io::{self, Write},
};

use crate::accept;

#[derive(Debug)]
pub enum Error {
    Accept(accept::Error),
    Recv(io::Error),
    ZeroBytesRecv(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Accept(err) => {
                write!(f, "failed to get accepted connection sock fd: {}", err)
            }
            Error::Recv(err) => write!(f, "recv err: {}", err),
            Error::ZeroBytesRecv(len) => {
                write!(f, "recv err: expected to read {} bytes, but read 0", len)
            }
        }
    }
}

impl error::Error for Error {}

impl From<accept::Error> for Error {
    fn from(value: accept::Error) -> Self {
        Self::Accept(value)
    }
}

// EXAMPLE: Receive a message from an accepted connection's socket.
// MANPAGE:
// man 2 recv (Linux)
// man 3 recv (POSIX)
pub fn recv() -> Result<(), Error> {
    let conn_sock_fd = crate::accept()?;

    let mut buf: Vec<u8> = vec![0; 30];
    let len = buf.len();

    // SAFETY:
    // 1 - `conn_sock_fd` contains an initialized sock fd when `accept()` succeeds.
    // 2 - Any potential `recv()` error is checked by reading `errno` instantly after the `recv()` call.
    // 3 - The `buf` passed to `recv()` is initialized.
    //
    // In addition, since receiving 0 bytes from `recv()` is not expected because the socket in example is of type SOCK_STREAM, `recv()` is accepted as failed if it does not read any bytes at all.
    let recv_bytes = unsafe {
        let bytes = libc::recv(conn_sock_fd, buf.as_mut_ptr() as *mut libc::c_void, len, 0);
        match bytes {
            -1 => {
                let err = io::Error::last_os_error();
                Err(Error::Recv(err))
            }
            0 => Err(Error::ZeroBytesRecv(len)),
            _ => Ok(bytes),
        }
    }?;

    let msg = [
        format!(
            "received {} bytes from sock fd {}: ",
            recv_bytes, conn_sock_fd
        )
        .as_bytes(),
        &buf,
    ]
    .concat();

    io::stdout()
        .write_all(&msg)
        .expect("received msg to be written to stdout");

    Ok(())
}
