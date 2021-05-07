use std::io;
use std::process;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fork {
    Parent,
    Child,
}

fn handle_err(res: i32) -> io::Result<i32> {
    match res {
        -1 => Err(io::Error::last_os_error()),
        r => Ok(r),
    }
}

fn fork() -> io::Result<Fork> {
    let r = unsafe { libc::fork() };
    handle_err(r).map(|r| match r {
        0 => Fork::Child,
        _ => Fork::Parent,
    })
}

fn detach_fds() -> io::Result<()> {
    handle_err(unsafe { libc::close(libc::STDOUT_FILENO) }).map(drop)?;
    handle_err(unsafe { libc::close(libc::STDERR_FILENO) }).map(drop)?;
    Ok(())
}

pub fn exec_child<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    match fork()? {
        Fork::Parent => Ok(()),
        Fork::Child => {
            detach_fds()?;
            let code = f().is_err() as i32;
            process::exit(code);
        }
    }
}
