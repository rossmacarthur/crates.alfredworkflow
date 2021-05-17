use std::io;
use std::panic::{self, PanicInfo};
use std::process;

use anyhow::Result;

use crate::index::logger::LOGGER;

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

fn close_fds() -> io::Result<()> {
    handle_err(unsafe { libc::close(libc::STDOUT_FILENO) })?;
    handle_err(unsafe { libc::close(libc::STDERR_FILENO) })?;
    handle_err(unsafe { libc::close(libc::STDIN_FILENO) })?;
    Ok(())
}

fn panic_hook(info: &PanicInfo<'_>) {
    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Box<Any>",
        },
    };
    log::error!("child panicked at '{}', {}", msg, info.location().unwrap());
}

fn execute<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    close_fds()?;
    log::set_logger(&*LOGGER)?;
    log::set_max_level(log::LevelFilter::Info);
    panic::set_hook(Box::new(panic_hook));
    f()?;
    Ok(())
}

/// Execute a function in a child process.
///
/// After a fork the following is done:
/// - In the parent:
///   - Returns immediately.
/// - In the child:
///   - Setup a logger that logs to a file.
///   - Setup a panic hook that logs an error on panic.
///   - Detach the stdin/stdout/stderr file descriptors.
pub fn child<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if let Fork::Child = fork()? {
        if let Err(err) = execute(f) {
            log::error!("{:#}", err);
            process::exit(1);
        }
        process::exit(0);
    }
    Ok(())
}
