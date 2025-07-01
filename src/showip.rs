use std::{
    error,
    ffi::{CStr, CString},
    fmt, mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    ptr,
};

#[derive(Debug)]
pub enum Error {
    Getaddrinfo(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
        }
    }
}

impl error::Error for Error {}

// EXAMPLE: Prints the IP address of the given host.
// Section 5.1 - `getaddrinfo()` - Prepare to Launch!
// MANPAGE: man 3 getaddrinfo
pub fn showip(host: &str) -> Result<(), Error> {
    let node = CString::new(host).unwrap();
    let node: *const libc::c_char = node.as_ptr();

    let port: *const libc::c_char = ptr::null();

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY: all the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    unsafe {
        let s = libc::getaddrinfo(node, port, &hints, &mut res_ptr);
        if s != 0 {
            let err = CStr::from_ptr(libc::gai_strerror(s)).to_string_lossy();
            return Err(Error::Getaddrinfo(err.into_owned()));
        }
    }

    println!("IP addresses for {}: \n\n", host);

    while !res_ptr.is_null() {
        // SAFETY: res_ptr is filled by getaddrinfo().
        let res = unsafe { *res_ptr };

        let addr = match res.ai_family as i32 {
            libc::AF_INET => {
                let sock_ipv4 = res.ai_addr as *const libc::sockaddr_in;
                // SAFETY: sock_ipv4 exists in res_ptr after getaddrinfo().
                let bits = unsafe { (*sock_ipv4).sin_addr.s_addr };

                IpAddr::V4(Ipv4Addr::from_bits(bits))
            }

            libc::AF_INET6 => {
                let sock_ipv6 = res.ai_addr as *const libc::sockaddr_in6;
                // SAFETY: sock_ipv6 exists in res_ptr after getaddrinfo().
                // sock_ipv6 encodes IPv6 (16 bytes) as fixed 16 length array containing each byte. Therefore, it is safe to call transmute().
                let bits = unsafe {
                    let addr = (*sock_ipv6).sin6_addr.s6_addr;
                    mem::transmute::<[u8; 16], u128>(addr)
                };

                IpAddr::V6(Ipv6Addr::from_bits(bits))
            }

            _ => unreachable!(),
        };

        let ipver = if addr.is_ipv4() { "IP" } else { "IPv6" };

        println!("{}: {:?}", ipver, addr);

        res_ptr = res.ai_next;
    }

    Ok(())
}
