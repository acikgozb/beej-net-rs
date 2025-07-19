use std::{
    ffi::CStr,
    io::{self, Write},
};

pub fn gethostname() -> Result<(), io::Error> {
    let mut host_buf: Vec<i8> = vec![0; 30];
    let len = host_buf.len();

    // SAFETY: `host_buf` is initialized. Accessing it is safe.
    let ecode = unsafe { libc::gethostname(host_buf.as_mut_ptr(), len) };
    match ecode {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(()),
    }?;

    // SAFETY: `host_buf` is initialized. Accessing it is safe.
    let host = unsafe { CStr::from_ptr(host_buf.as_ptr() as _) };

    let msg = [b"hostname: ", host.to_bytes()].concat();
    io::stdout()
        .write_all(&msg)
        .expect("message to be written to stdout");

    Ok(())
}
