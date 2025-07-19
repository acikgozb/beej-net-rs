use std::{io, ptr};

// EXAMPLE: Poll stdin to see whether it is ready to be read or not.
// MANPAGE:
// man 2 poll (Linux)
// man 3 poll (POSIX)
// man errno
pub fn poll() -> Result<(), io::Error> {
    let mut pfds = [libc::pollfd {
        fd: 0,                // stdin
        events: libc::POLLIN, // notify when fd is ready to be read
        revents: 0,           // will be filled by `poll()`.
    }];

    let pfds_ptr = ptr::addr_of_mut!(pfds);

    println!("Hit RETURN or wait 2.5 seconds for timeout");

    const POLL_TIMEOUT: i32 = 2500;
    let num_events = unsafe {
        libc::poll(
            pfds_ptr as *mut libc::pollfd,
            pfds.len() as u64,
            POLL_TIMEOUT,
        )
    };
    match num_events {
        -1 => Err(io::Error::last_os_error()),
        0 => {
            println!("Poll timed out!");
            Ok(())
        }
        _ => {
            let pollin_happened = (pfds[0].revents & libc::POLLIN) == 1;
            if pollin_happened {
                let fd = pfds[0].fd;
                println!("File descriptor {} is ready to read", fd);
            } else {
                println!("Unexpected event occurred: {}", pfds[0].revents);
            }
            Ok(())
        }
    }
}
