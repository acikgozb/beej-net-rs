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
    Bind(i32, io::Error),
    Listen(i32, io::Error),
    InvalidAddressFamily,
    Setsockopt(io::Error),
    Select(io::Error),
    Accept(io::Error),
    Recv(i32, io::Error),
    Send(i32, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Getaddrinfo(err) => write!(f, "getaddrinfo error: {}", err),
            Error::Socket(err) => write!(f, "socket error: {}", err),
            Error::Setsockopt(err) => write!(f, "setsockopt error: {}", err),
            Error::Bind(sock_fd, err) => write!(f, "bind error for sock fd {}: {}", sock_fd, err),
            Error::Listen(sock_fd, err) => {
                write!(f, "listen error for sock fd {}: {}", sock_fd, err)
            }
            Error::InvalidAddressFamily => write!(
                f,
                "ip conv failed: the given address family is not AF_INET or AF_INET6"
            ),
            Error::Select(err) => write!(f, "select error: {}", err),
            Error::Accept(err) => write!(f, "accept error: {}", err),
            Error::Recv(sock_fd, err) => write!(f, "recv error on sock fd {}: {}", sock_fd, err),
            Error::Send(sock_fd, err) => write!(f, "send error on sock fd {}: {}", sock_fd, err),
        }
    }
}

impl error::Error for Error {}

struct FdSet {
    master_set: libc::fd_set,
    op_set: libc::fd_set,
    max_fd: i32,
}

impl FdSet {
    pub fn new(listener_fd: i32) -> Self {
        // SAFETY: Zeroed master and op sets are initialized
        // correctly with `FD_ZERO`.
        // It is safe to use the sets for the lifetime of Self.
        let (master_set, op_set) = unsafe {
            let mut master_set: libc::fd_set = mem::zeroed();
            libc::FD_ZERO(&mut master_set);
            libc::FD_SET(listener_fd, &mut master_set);

            let mut op_set: libc::fd_set = mem::zeroed();
            libc::FD_ZERO(&mut op_set);

            (master_set, op_set)
        };

        Self {
            master_set,
            op_set,
            max_fd: listener_fd,
        }
    }

    pub fn max_fd(&self) -> i32 {
        self.max_fd
    }

    pub fn as_mut(&mut self) -> &mut libc::fd_set {
        self.op_set = self.master_set;
        &mut self.op_set
    }

    pub fn iter_sfd(&self) -> impl Iterator<Item = i32> {
        // SAFETY: `self.op_set` is initialized correctly.
        // It is safe to call `FD_ISSET`.
        (0..=self.max_fd).filter(|fd| unsafe { libc::FD_ISSET(*fd, &self.op_set) })
    }

    pub fn iter_fd(&self) -> impl Iterator<Item = i32> {
        // SAFETY: `self.master_set` is initialized correctly.
        // It is safe to call `FD_ISSET`.
        (0..=self.max_fd).filter(|fd| unsafe { libc::FD_ISSET(*fd, &self.master_set) })
    }

    pub fn apply_changes(&mut self, changes: &[SfdChange]) {
        for change in changes {
            match change {
                SfdChange::Add(fd) => {
                    // SAFETY: `self.master_set` is initialized correctly
                    // for each instance of `Self`, making `FD_SET` safe to call.
                    unsafe {
                        libc::FD_SET(*fd, &mut self.master_set);
                    };

                    if *fd > self.max_fd {
                        self.max_fd = *fd;
                    }
                }
                // SAFETY: `self.master_set` is initialized correctly
                // for each instance of `Self`, making `FD_CLR` safe to call.
                SfdChange::Remove(fd) => unsafe { libc::FD_CLR(*fd, &mut self.master_set) },
            }
        }
    }
}

enum SfdChange {
    Add(i32),
    Remove(i32),
}

const RECV_MESSAGE_SIZE: usize = 256;

// EXAMPLE: A multiperson chat server.
// This example is a more complete version of the `select()` syscall example.
// MANPAGE:
// man 2 select
// man errno
pub fn selectserver() -> Result<(), Error> {
    let listener_fd = setup_listener_socket()?;
    let mut fds = FdSet::new(listener_fd);

    loop {
        // SAFETY: The fd set for read operations is correctly
        // initialized via `FdSet::new()`.
        // The remaining sets for other operations are intentionally set as null.
        // There are no uninitialized reads during `select()`.
        // It is safe to call.
        let ecode = unsafe {
            libc::select(
                fds.max_fd() + 1,
                fds.as_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if ecode == -1 {
            let err = io::Error::last_os_error();
            Err(Error::Select(err))?;
        }

        let mut changes: Vec<SfdChange> = vec![];
        for sfd in fds.iter_sfd() {
            if sfd == listener_fd {
                let client_fd = accept_new_client(listener_fd);
                changes.push(SfdChange::Add(client_fd));
                continue;
            }

            let (closed_fd, msg_buf, rbytes) = recv_client_message(sfd);
            if let Some(fd) = closed_fd {
                changes.push(SfdChange::Remove(fd));
                continue;
            }

            let dest_fds = fds.iter_fd().filter(|fd| *fd != listener_fd && *fd != sfd);

            broadcast_message(msg_buf, rbytes, dest_fds);
        }

        fds.apply_changes(&changes);
    }
}

fn broadcast_message(
    buf: [u8; RECV_MESSAGE_SIZE],
    nbytes: isize,
    dest_fds: impl Iterator<Item = i32>,
) {
    for fd in dest_fds {
        // SAFETY: A readonly reference to `buf` is used for
        // each iteration.
        // `buf` is valid for the entire duration of the iteration.
        // There are no uninitialized reads on `buf`.
        // Therefore, it is safe to call `send()`.
        let sbytes =
            unsafe { libc::send(fd, buf.as_ptr() as *const libc::c_void, nbytes as usize, 0) };
        if sbytes == -1 {
            eprintln!("{}", Error::Send(fd, io::Error::last_os_error()));
        }
    }
}

fn recv_client_message(source_fd: i32) -> (Option<i32>, [u8; 256], isize) {
    let mut recv_buf = [0; RECV_MESSAGE_SIZE];
    let len = recv_buf.len();

    // SAFETY: There are no uninitialized reads on `source_fd`, `recv_buf` and `len`.
    // It is safe to call `recv()`.
    let nbytes = unsafe {
        libc::recv(
            source_fd,
            recv_buf.as_mut_ptr() as *mut libc::c_void,
            len,
            0,
        )
    };
    match nbytes {
        n if n <= 0 => {
            if n == 0 {
                println!("selectserver: socket {} hung up", source_fd);
            } else {
                eprintln!("{}", Error::Recv(source_fd, io::Error::last_os_error()));
            }

            // SAFETY: `source_fd` is not used after a failed `recv()` attempt.
            // Therefore, `close()` is safe to call.
            unsafe { libc::close(source_fd) };

            (Some(source_fd), recv_buf, n)
        }
        n => (None, recv_buf, n),
    }
}

fn accept_new_client(listener_fd: i32) -> i32 {
    // SAFETY: A full zeroed `sockaddr_storage` will be initialized
    // correctly upon a successful `accept()` call.
    // Upon a failure, it is not read.
    // Therefore it is safe to initialize it like this.
    let mut client_addr: libc::sockaddr_storage = unsafe { mem::zeroed() };
    let mut len = mem::size_of_val(&client_addr);

    // SAFETY: All required variables are initialized correctly.
    // `accept()` is safe to call.
    let client_fd = unsafe {
        libc::accept(
            listener_fd,
            &raw mut client_addr as *mut libc::sockaddr,
            &raw mut len as *mut u32,
        )
    };
    if client_fd == -1 {
        eprintln!("{}", Error::Accept(io::Error::last_os_error()));
    }

    // SAFETY: It is safe to cast `sockaddr_storage` to `sockaddr` upon a successful `accept()` call.
    let sa_client = unsafe { *(&raw const client_addr as *const libc::sockaddr) };
    match try_into_ip_addr(sa_client) {
        Some(ip_addr) => println!(
            "selectserver: new connection from {} on socket {}",
            ip_addr, client_fd
        ),
        None => eprintln!("{}", Error::InvalidAddressFamily),
    }

    client_fd
}

fn setup_listener_socket() -> Result<i32, Error> {
    let node = ptr::null();
    let port = CString::from(c"9034");

    // SAFETY: All zero hints is a valid initialization.
    // Required fields are set later on.
    let mut hints: libc::addrinfo = unsafe { mem::zeroed() };
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_STREAM;
    hints.ai_flags = libc::AI_PASSIVE;

    let mut gai_res_ptr: *mut libc::addrinfo = ptr::null_mut();

    // SAFETY: There are no uninitialized reads. `getaddrinfo()` is safe to use.
    let ecode = unsafe { libc::getaddrinfo(node, port.as_ptr(), &hints, &mut gai_res_ptr) };
    if ecode != 0 {
        // SAFETY: `gai_strerror` is valid to call on a failed `getaddrinfo()` call.
        let err = unsafe { CStr::from_ptr(libc::gai_strerror(ecode)) }.to_string_lossy();
        return Err(Error::Getaddrinfo(err.into_owned()));
    };

    let mut listener_sockaddr: *mut libc::sockaddr = ptr::null_mut();
    let mut listener_fd = -1;

    while !gai_res_ptr.is_null() {
        // SAFETY: `gai_res_ptr` is guaranteed to point atleast one valid addrinfo struct on a successful `getaddrinfo()` call.
        let ai = unsafe { *gai_res_ptr };
        let next_ai_ptr = ai.ai_next;

        // SAFETY: `socket()` is safe to call since `ai` is valid.
        let sock_fd = unsafe { libc::socket(ai.ai_family, ai.ai_socktype, 0) };
        if sock_fd == -1 {
            if next_ai_ptr.is_null() {
                let err = io::Error::last_os_error();
                return Err(Error::Socket(err));
            } else {
                gai_res_ptr = next_ai_ptr;
                continue;
            }
        }

        let yes = 1;
        let len = mem::size_of::<i32>();
        // SAFETY: `setsockopt()` is called for a valid sock_fd created by a successful `socket()` call, making it safe to use.
        let ecode = unsafe {
            libc::setsockopt(
                sock_fd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &raw const yes as *const libc::c_void,
                len as u32,
            )
        };
        if ecode == -1 {
            if next_ai_ptr.is_null() {
                let err = io::Error::last_os_error();
                return Err(Error::Setsockopt(err));
            } else {
                gai_res_ptr = next_ai_ptr;
                continue;
            }
        }

        // SAFETY: The socket and address used for `bind()` are valid due to `socket()` and `getaddrinfo()` calls above.
        // Bind is safe to call.
        let ecode = unsafe { libc::bind(sock_fd, ai.ai_addr, ai.ai_addrlen) };
        if ecode == -1 {
            if next_ai_ptr.is_null() {
                let err = io::Error::last_os_error();
                return Err(Error::Bind(sock_fd, err));
            } else {
                gai_res_ptr = next_ai_ptr;
                continue;
            }
        }

        listener_sockaddr = ai.ai_addr;
        listener_fd = sock_fd;
        break;
    }

    const BACKLOG: i32 = 10;
    // SAFETY: `listener_fd` is a valid fd obtained above through `socket()`.
    // It is safe to call `listen()`.
    let ecode = unsafe { libc::listen(listener_fd, BACKLOG) };
    if ecode == -1 {
        let err = io::Error::last_os_error();
        return Err(Error::Listen(listener_fd, err));
    }

    // SAFETY: `listener_sockaddr` is filled by a successful `getaddrinfo()`
    // call and is valid to read.
    let sa = unsafe { *listener_sockaddr };
    let ip_addr = try_into_ip_addr(sa).ok_or(Error::InvalidAddressFamily)?;

    println!(
        "server is listening on {} port {}",
        ip_addr,
        port.to_str().unwrap()
    );

    // SAFETY: The `getaddrinfo()` response is not used from now on.
    // It is safe to free the allocated memory for `getaddrinfo()`.
    unsafe {
        libc::freeaddrinfo(gai_res_ptr);
    }

    Ok(listener_fd)
}

fn try_into_ip_addr(sa: libc::sockaddr) -> Option<IpAddr> {
    match sa.sa_family as i32 {
        libc::AF_INET => {
            // SAFETY: For `AF_INET`, it is safe to cast the `sockaddr` container to `sockaddr_in`.
            let sockaddr_in = unsafe { *(&raw const sa as *const libc::sockaddr_in) };
            let bits = u32::from_be(sockaddr_in.sin_addr.s_addr);
            let inet = Ipv4Addr::from_bits(bits);
            Some(IpAddr::V4(inet))
        }
        libc::AF_INET6 => {
            // SAFETY: For `AF_INET6`, it is safe to cast the `sockaddr` container to `sockaddr_in6`.
            let sockaddr_in6 = unsafe { *(&raw const sa as *const libc::sockaddr_in6) };
            let bits = u128::from_be_bytes(sockaddr_in6.sin6_addr.s6_addr);
            let inet6 = Ipv6Addr::from_bits(bits);
            Some(IpAddr::V6(inet6))
        }
        _ => None,
    }
}
