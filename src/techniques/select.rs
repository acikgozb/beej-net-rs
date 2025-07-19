use core::fmt;
use std::{error, io, mem, os::fd::AsRawFd, ptr};

#[derive(Debug)]
pub enum Error {
    Select(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Select(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Select(err) => write!(f, "select error: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Wait 2.5 seconds for something to appear on standard input.
// MANPAGE:
// man 2 select
pub fn select() -> Result<(), Error> {
    let stdin_fd = io::stdin().as_raw_fd();

    // SAFETY: Whilst `readfds` is initialized as zeroed,
    // the struct is correctly filled with the macros.
    // It is safe to read.
    let mut readfds = unsafe {
        let mut readfds = mem::zeroed();
        libc::FD_ZERO(&mut readfds);
        libc::FD_SET(stdin_fd, &mut readfds);

        readfds
    };

    let mut timeval = libc::timeval {
        tv_sec: 2,
        tv_usec: 500000,
    };

    // SAFETY: The required set is initialized properly,
    // and the rest is set to NULL as desired.
    // `select` is safe to use.
    let ecode = unsafe {
        libc::select(
            stdin_fd + 1,
            &mut readfds,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut timeval,
        )
    };
    if ecode == -1 {
        let err = io::Error::last_os_error();
        return Err(err.into());
    }

    // SAFETY: `readfs` is read and written by a successful `select()` call.
    // It is safe to read.
    let stdin_isset = unsafe { libc::FD_ISSET(stdin_fd, &readfds) };

    if stdin_isset {
        println!("A key was pressed!");
    } else {
        println!("Timed out.");
    }

    Ok(())
}
