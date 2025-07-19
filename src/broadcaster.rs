use std::{
    error, fmt,
    io::{self},
    mem,
    net::{AddrParseError, Ipv4Addr},
    str::FromStr,
};

#[derive(Debug)]
pub enum Error {
    Socket(io::Error),
    InvalidInetAddr(AddrParseError),
    Sendto(io::Error),
    Setsockopt(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::InvalidInetAddr(err) => write!(f, "failed to parse host IP addr: {}", err),
            Error::Sendto(err) => write!(f, "sendto error: {}", err),
            Error::Setsockopt(err) => write!(f, "setsockopt error: {}", err),
        }
    }
}

impl error::Error for Error {}

impl From<AddrParseError> for Error {
    fn from(value: AddrParseError) -> Self {
        Self::InvalidInetAddr(value)
    }
}

// EXAMPLE: Broadcast a UDP message to all hosts on a network.
// MANPAGE:
// man 2 setsockopt
// man 7 socket
// man errno
pub fn broadcaster(host: String, msg: String) -> Result<(), Error> {
    let host_ip_addr = Ipv4Addr::from_str(&host)?;

    // SAFETY: Hardcoded opts are used: An INET DGRAM sock.
    // `socket()` is safe to call.
    let sock_fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if sock_fd == -1 {
        Err(Error::Socket(io::Error::last_os_error()))?;
    }

    let broadcast = 1;
    // SAFETY: `sock_fd` is ensured to be a valid sock fd.
    // There are no uninitialized reads in here.
    // `setsockopt()` is safe to call.
    let ecode = unsafe {
        libc::setsockopt(
            sock_fd,
            libc::SOL_SOCKET,
            libc::SO_BROADCAST,
            &raw const broadcast as *const libc::c_void,
            mem::size_of::<i32>() as u32,
        )
    };
    if ecode == -1 {
        Err(Error::Setsockopt(io::Error::last_os_error()))?;
    }

    let port: u16 = 4950;

    // SAFETY: The required fields are set to initialize a valid
    // `sockaddr_in`.
    // `sockaddr_in.sin_zero` is left as full zeroes, which is valid
    // for a padding field.
    // It is safe to read from `sa_host`.
    let mut sa_host: libc::sockaddr_in = unsafe { mem::zeroed() };
    sa_host.sin_family = libc::AF_INET as u16;
    sa_host.sin_port = u16::from_be(port);
    sa_host.sin_addr.s_addr = u32::from_be(host_ip_addr.to_bits());

    // SAFETY: All variables are initialized properly.
    // `sendto()` is safe to call.
    let sbytes = unsafe {
        libc::sendto(
            sock_fd,
            msg.as_ptr() as *const libc::c_void,
            msg.len(),
            0,
            &raw const sa_host as *const libc::sockaddr,
            mem::size_of_val(&sa_host) as u32,
        )
    };
    if sbytes == -1 {
        Err(Error::Sendto(io::Error::last_os_error()))?;
    }

    println!("sent {} bytes to {}", sbytes, host_ip_addr);

    // SAFETY: We have no use for `sock_fd` at this point.
    // It is safe to close.
    unsafe { libc::close(sock_fd) };

    Ok(())
}
