use std::{
    error,
    ffi::{CStr, CString},
    fmt,
    io::{self, Write},
    mem,
    net::Ipv6Addr,
    ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Bind(io::Error),
    Recvfrom(io::Error),
    InvalidAddrFamily(i32),
    Close(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Bind(err) => write!(f, "bind error: {}", err),
            Error::Recvfrom(err) => write!(f, "recvfrom error: {}", err),
            Error::InvalidAddrFamily(af) => write!(f, "recvfrom error: invalid addr family {}", af),
            Error::Close(err) => write!(f, "close error: {}", err),
        }
    }
}
impl error::Error for Error {}

// EXAMPLE: A DGRAM socket listener that receives UDP messages.
// This example is a more complete version of `recvfrom()` syscall.
// MANPAGE:
// man 2 recvfrom (Linux)
// man 2 recvfrom (POSIX)
// man errno
pub fn server() -> Result<(), Error> {
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
        // SAFETY: `gai_res_ptr` is guaranteed to point atleast one valid addrinfo struct on a successful `getaddrinfo()` call.
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

        // SAFETY: `bind()` is safe to call since `sock` and `gai_res` are valid.
        let ecode = unsafe { libc::bind(sock, gai_res.ai_addr, gai_res.ai_addrlen) };
        if ecode == -1 {
            if next_res_ptr.is_null() {
                return Err(Error::Bind(io::Error::last_os_error()));
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

    println!("listener: waiting to recvfrom...");

    const MAXBUFLEN: usize = 100;
    let mut recv_buf = vec![0; MAXBUFLEN];
    let len = recv_buf.len();

    // SAFETY: All zero `sockaddr_storage` is a valid initialization.
    // Read will happen after it is written by `recvfrom()`.
    let mut sockaddr: libc::sockaddr_storage = unsafe { mem::zeroed() };
    let mut sa_len = mem::size_of_val(&sockaddr) as u32;

    // SAFETY:
    // 1 - `sock_fd` is a valid socket.
    // 2 - The buf is initialized as desired.
    // 3 - Casting `sockaddr_storage` to `sockaddr` is valid and expected.
    let bytes = unsafe {
        libc::recvfrom(
            sock_fd,
            recv_buf.as_mut_ptr() as *mut libc::c_void,
            len,
            0,
            &raw mut sockaddr as *mut libc::sockaddr,
            &raw mut sa_len,
        )
    };
    match bytes {
        -1 => Err(Error::Recvfrom(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    let sockaddr = match sockaddr.ss_family as i32 {
        libc::AF_INET6 => {
            // SAFETY: If `ss_family` is INET6, and we know it is due to `getaddrinfo()`, then `sockaddr_storage` can be casted safely to `sockaddr_in6` to access the data written by `recvfrom()`.
            let sockaddr_in6 = unsafe { *(&raw const sockaddr as *const libc::sockaddr_in6) };
            Ok(sockaddr_in6)
        }
        af => Err(Error::InvalidAddrFamily(af)),
    }?;
    let ip_addr = {
        let bits = u128::from_be_bytes(sockaddr.sin6_addr.s6_addr);
        Ipv6Addr::from_bits(bits)
    };

    println!("listener: got packet from {}", ip_addr);
    println!("listener: packet is {} bytes long", bytes);

    recv_buf[bytes as usize] = b'\0';

    let msg = [b"listener: packet contains ", &recv_buf[..]].concat();
    io::stdout()
        .write_all(&msg)
        .expect("message to be written to stdout");

    // SAFETY: The communication has ended. It is safe to close the socket.
    let ecode = unsafe { libc::close(sock_fd) };
    match ecode {
        -1 => Err(Error::Close(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    Ok(())
}
