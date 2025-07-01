use std::ffi::CString;
use std::{mem, ptr};
mod showip;

pub use showip::showip;

// EXAMPLE: Showcases how socket() can be used.
// Section 5.2 - `socket()` - Get the File Descriptor!
// MANPAGE: man 3 socket
pub fn socket() -> Result<i32, i32> {
    let node = CString::new("www.example.com").unwrap();
    let node_ptr = node.as_ptr();

    let service = CString::new("http").unwrap();
    let service_ptr = service.as_ptr();

    // SAFETY: hints is initialized as empty, but the required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY: all the required vars are initialized for getaddrinfo().
    // gai_stderror() is used for error cases only.
    unsafe {
        let s = libc::getaddrinfo(node_ptr, service_ptr, &hints, &mut res_ptr);
        if s != 0 {
            eprintln!("getaddrinfo error: {}", *libc::gai_strerror(s));
            return Err(s);
        }
    }

    unsafe {
        let res = *res_ptr;
        let sock_fd = libc::socket(res.ai_family, res.ai_socktype, res.ai_protocol);
        if sock_fd == -1 {
            eprintln!("socket error: {}", *libc::strerror(sock_fd));
            return Err(sock_fd);
        }

        println!("created sock fd: {}", sock_fd);

        libc::freeaddrinfo(res_ptr);

        Ok(sock_fd)
    }
}
