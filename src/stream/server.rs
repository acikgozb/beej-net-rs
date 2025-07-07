use std::{
    error,
    ffi::{CStr, CString},
    fmt, io, mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
    Socket(io::Error),
    Setsockopt(io::Error),
    Bind(io::Error),
    Listen(io::Error),
    Accept(io::Error),
    InvalidAddrFamily(i32),
    Send(io::Error),
    Close(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Setsockopt(err) => write!(f, "setsockopt error: {}", err),
            Error::Bind(err) => write!(f, "bind error: {}", err),
            Error::Listen(err) => write!(f, "listen error: {}", err),
            Error::Accept(err) => write!(f, "accept error: {}", err),
            Error::Send(err) => write!(f, "send error: {}", err),
            Error::InvalidAddrFamily(af) => {
                write!(f, "accept error: invalid address family {}", af)
            }
            Error::Close(err) => write!(f, "close error: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: A simple stream server that sends "Hello world!" to a connected peer.
// This example is a more complete version of `send()` syscall example.
// MANPAGE:
// man 2 send (Linux)
// man 3 send (POSIX)
// man errno
pub fn server() -> Result<(), Error> {
    let node = ptr::null();
    let port = CString::from(c"3490");

    // SAFETY: All zero hints is a valid initialization.
    // Required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;

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

    // traverse the linked list and find a proper addr that can be used as a sock and bind
    let mut sock_fd = -1;
    while !gai_res_ptr.is_null() {
        let gai_res = unsafe { *gai_res_ptr };
        let next_res_ptr = gai_res.ai_next;

        let sock = unsafe { libc::socket(gai_res.ai_family, gai_res.ai_socktype, 0) };
        if sock == -1 {
            if next_res_ptr.is_null() {
                return Err(Error::Socket(io::Error::last_os_error()));
            } else {
                gai_res_ptr = next_res_ptr;
                continue;
            }
        }

        let reuse_sock = 1;
        let size = mem::size_of_val(&reuse_sock);
        let ecode = unsafe {
            libc::setsockopt(
                sock,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &raw const reuse_sock as _,
                size as libc::socklen_t,
            )
        };
        if ecode == -1 {
            return Err(Error::Setsockopt(io::Error::last_os_error()));
        }

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

    // SAFETY: `listen()` is safe to use on a valid `sock_fd`.
    let ecode = unsafe { libc::listen(sock_fd, 10) };
    match ecode {
        -1 => Err(Error::Listen(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    println!("server: waiting for connections...");

    loop {
        // SAFETY:
        // 1 - All zeroed `sockaddr_storage` is a valid initialization.
        // 2 - `sock_fd` a valid socket fd.
        let (conn_sock_fd, sockaddr) = unsafe {
            let mut sockaddr: libc::sockaddr_storage = mem::zeroed();
            let mut len = mem::size_of_val(&sockaddr);

            let conn_sock_fd = libc::accept(
                sock_fd,
                &raw mut sockaddr as *mut libc::sockaddr,
                &raw mut len as *mut _,
            );

            (conn_sock_fd, sockaddr)
        };
        match conn_sock_fd {
            -1 => Err(Error::Accept(io::Error::last_os_error())),
            _ => Ok(()),
        }?;

        // SAFETY:
        // 1 - `sockaddr_storage` pointer points to a memory that is initialized by a successful `accept()` call.
        // 2 - raw `sockaddr_storage` pointer is casted to INET or INET6 based on the address family filled by `accept()`.
        let from_addr = unsafe {
            match sockaddr.ss_family as i32 {
                libc::AF_INET => {
                    let sockaddr_in = *(&raw const sockaddr as *const libc::sockaddr_in);

                    let bits = u32::from_be(sockaddr_in.sin_addr.s_addr);
                    Ok(IpAddr::V4(Ipv4Addr::from_bits(bits)))
                }
                libc::AF_INET6 => {
                    let sockaddr_in6 = *(&raw const sockaddr as *const libc::sockaddr_in6);

                    let bits = u128::from_be_bytes(sockaddr_in6.sin6_addr.s6_addr);
                    Ok(IpAddr::V6(Ipv6Addr::from_bits(bits)))
                }
                af => Err(Error::InvalidAddrFamily(af)),
            }
        }?;
        println!("server: got connection from {}", from_addr);

        let msg = b"Hello world!\n";
        let len = msg.len();

        // SAFETY:
        // 1 - `conn_sock_fd` is a valid sock fd for peer communication.
        // 2 - The message and its len are initialized as desired.
        let bytes =
            unsafe { libc::send(conn_sock_fd, msg.as_ptr() as *const libc::c_void, len, 0) };
        match bytes {
            -1 => Err(Error::Send(io::Error::last_os_error())),
            _ => Ok(()),
        }?;

        // SAFETY:
        // `conn_sock_fd` is a valid sock fd for peer communication.
        let ecode = unsafe { libc::close(conn_sock_fd) };
        match ecode {
            -1 => Err(Error::Close(io::Error::last_os_error())),
            _ => Ok(()),
        }?;
    }
}
