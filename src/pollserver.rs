use std::{
    error,
    ffi::{CStr, CString},
    fmt,
    io::{self, Write},
    mem,
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
    Poll(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Setsockopt(err) => write!(f, "setsockopt error: {}", err),
            Error::Bind(err) => write!(f, "bind error: {}", err),
            Error::Listen(err) => write!(f, "listen error: {}", err),
            Error::Poll(err) => write!(f, "poll error: {}", err),
        }
    }
}

impl error::Error for Error {}

struct Pfds {
    pfds: Vec<libc::pollfd>,
}

impl Pfds {
    pub fn new(listener_fd: i32) -> Self {
        const FD_SIZE: usize = 5;
        let mut pfds = Vec::with_capacity(FD_SIZE);

        let listener_pfd = libc::pollfd {
            fd: listener_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        pfds.push(listener_pfd);

        Self { pfds }
    }

    pub fn len(&self) -> usize {
        self.pfds.len()
    }

    pub fn as_mut_ptr(&mut self) -> *mut libc::pollfd {
        self.pfds.as_mut_ptr()
    }

    pub fn iter(&self) -> impl Iterator<Item = &libc::pollfd> {
        self.pfds.iter()
    }

    pub fn apply_changes(&mut self, ops: &[PfdChange]) {
        for op in ops {
            match op {
                PfdChange::Remove(fd) => {
                    let idx = self.pfds.iter().position(|pfd| pfd.fd == *fd);
                    if let Some(idx) = idx {
                        self.pfds.swap_remove(idx);
                    }
                }
                PfdChange::Insert(fd) => {
                    let pfd = libc::pollfd {
                        fd: *fd,
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    self.pfds.push(pfd);
                }
            }
        }
    }
}

enum PfdChange {
    Remove(i32),
    Insert(i32),
}

// EXAMPLE: A multiperson chat server.
// This example is a more complete version of the `poll()` syscall example.
// MANPAGE:
// man 2 poll (Linux)
// man 3 poll (POSIX)
// man errno
pub fn pollserver() -> Result<(), Error> {
    let listener_fd = get_listener_socket()?;
    let mut pfds = Pfds::new(listener_fd);

    println!("pollserver: waiting for connections...");

    loop {
        let poll_count = unsafe { libc::poll(pfds.as_mut_ptr(), pfds.len() as u64, -1) };
        match poll_count {
            -1 => Err(Error::Poll(io::Error::last_os_error())),
            _ => Ok(()),
        }?;

        let changes = process_connections(listener_fd, &pfds);
        pfds.apply_changes(&changes);
    }
}

fn get_listener_socket() -> Result<i32, Error> {
    let port = CString::from(c"9034");

    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_INET;
    hints.ai_socktype = libc::SOCK_STREAM;

    let mut gai_res_ptr: *mut libc::addrinfo = ptr::null_mut();

    let ecode = unsafe { libc::getaddrinfo(ptr::null(), port.as_ptr(), &hints, &mut gai_res_ptr) };
    match ecode {
        0 => Ok(()),
        _ => {
            let err = unsafe { CStr::from_ptr(libc::gai_strerror(ecode)).to_string_lossy() };
            Err(Error::Getaddrinfo(err.into_owned()))
        }
    }?;

    let mut sock_fd = -1;

    while !gai_res_ptr.is_null() {
        let ai = unsafe { *gai_res_ptr };
        let next_ai_ptr = ai.ai_next;

        let sock = unsafe { libc::socket(ai.ai_family, ai.ai_socktype, 0) };
        if sock == -1 {
            if next_ai_ptr.is_null() {
                return Err(Error::Socket(io::Error::last_os_error()));
            } else {
                gai_res_ptr = next_ai_ptr;
                continue;
            }
        }

        let yes: i32 = 1;
        let ecode = unsafe {
            libc::setsockopt(
                sock,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &raw const yes as *const libc::c_void,
                mem::size_of::<i32>() as u32,
            )
        };
        if ecode == -1 {
            if next_ai_ptr.is_null() {
                return Err(Error::Setsockopt(io::Error::last_os_error()));
            } else {
                gai_res_ptr = next_ai_ptr;
                continue;
            }
        }

        let ecode = unsafe { libc::bind(sock, ai.ai_addr, ai.ai_addrlen) };
        if ecode == -1 {
            if next_ai_ptr.is_null() {
                return Err(Error::Bind(io::Error::last_os_error()));
            } else {
                gai_res_ptr = next_ai_ptr;
                continue;
            }
        }

        sock_fd = sock;
        break;
    }

    unsafe { libc::freeaddrinfo(gai_res_ptr) };

    const BACKLOG: i32 = 10;
    let ecode = unsafe { libc::listen(sock_fd, BACKLOG) };
    match ecode {
        -1 => Err(Error::Listen(io::Error::last_os_error())),
        _ => Ok(()),
    }?;

    Ok(sock_fd)
}

fn process_connections(listener_fd: i32, pfds: &Pfds) -> Vec<PfdChange> {
    let mut changes = vec![];

    let source_fds = pfds.iter().filter_map(|pfd| {
        if (pfd.revents & (libc::POLLIN | libc::POLLHUP)) == 1 {
            Some(pfd.fd)
        } else {
            None
        }
    });

    for source_fd in source_fds {
        if source_fd == listener_fd {
            let client_fd = accept_new_client(listener_fd);
            changes.push(PfdChange::Insert(client_fd));
        } else {
            let dest_fds = pfds.iter().filter_map(|pfd| {
                if pfd.fd != source_fd && pfd.fd != listener_fd {
                    Some(pfd.fd)
                } else {
                    None
                }
            });
            let closed_fd = send_message_to_clients(source_fd, dest_fds);
            if let Some(fd) = closed_fd {
                changes.push(PfdChange::Remove(fd))
            }
        }
    }

    changes
}

fn accept_new_client(sock_fd: i32) -> i32 {
    let mut sockaddr: libc::sockaddr_storage = unsafe { mem::zeroed() };
    let mut len = mem::size_of_val(&sockaddr);

    let (conn_sock_fd, sockaddr) = unsafe {
        let sock = libc::accept(
            sock_fd,
            &raw mut sockaddr as *mut libc::sockaddr,
            &raw mut len as *mut libc::socklen_t,
        );
        (sock, sockaddr)
    };
    if conn_sock_fd == -1 {
        eprintln!("accept error: {}", io::Error::last_os_error());
    }

    let ip_addr = try_into_ip_addr(sockaddr);
    if let Some(ip_addr) = ip_addr {
        println!(
            "pollserver: new connection from {} on socket {}",
            ip_addr, conn_sock_fd
        );
    }

    conn_sock_fd
}

fn send_message_to_clients(source_fd: i32, dest_fds: impl Iterator<Item = i32>) -> Option<i32> {
    let mut recv_buf = vec![0; 256];
    let len = recv_buf.len();

    let bytes = unsafe {
        libc::recv(
            source_fd,
            recv_buf.as_mut_ptr() as *mut libc::c_void,
            len,
            0,
        )
    };

    if bytes <= 0 {
        if bytes < 0 {
            eprintln!("pollserver: recv error: {}", io::Error::last_os_error());
        }
        eprintln!("pollserver: socket {} hung up", source_fd);

        unsafe { libc::close(source_fd) };

        Some(source_fd)
    } else {
        let msg = [
            format!("pollserver: recv from fd {}: ", source_fd).as_bytes(),
            &recv_buf[..],
        ]
        .concat();
        io::stdout()
            .write_all(&msg)
            .expect("message to be written to stdout");

        for fd in dest_fds {
            let bytes: usize = bytes.try_into().unwrap();

            let ecode =
                unsafe { libc::send(fd, recv_buf.as_mut_ptr() as *const libc::c_void, bytes, 0) };
            if ecode == -1 {
                eprintln!("pollserver: send error: {}", io::Error::last_os_error());
            };
        }

        None
    }
}

fn try_into_ip_addr(sockaddr: libc::sockaddr_storage) -> Option<IpAddr> {
    match sockaddr.ss_family as i32 {
        libc::AF_INET => {
            let sockaddr_in = unsafe { *(&raw const sockaddr as *const libc::sockaddr_in) };
            let bits = u32::from_be(sockaddr_in.sin_addr.s_addr);
            Some(IpAddr::V4(Ipv4Addr::from_bits(bits)))
        }
        libc::AF_INET6 => {
            let sockaddr_in6 = unsafe { *(&raw const sockaddr as *const libc::sockaddr_in6) };
            let bits = u128::from_be_bytes(sockaddr_in6.sin6_addr.s6_addr);
            Some(IpAddr::V6(Ipv6Addr::from_bits(bits)))
        }
        af => {
            eprintln!("pollserver: invalid address family {}", af);
            None
        }
    }
}
