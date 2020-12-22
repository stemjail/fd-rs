// Copyright (C) 2015-2017 Mickaël Salaün
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, version 3 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

#![cfg(not(target_os = "macos"))]

use libc::{size_t, ssize_t};
use std::{io, ptr};
use std::os::unix::io::RawFd;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::Sender;

mod raw {
    use std::os::unix::io::RawFd;
    use ::libc::{c_longlong, size_t, ssize_t, c_uint};

    pub use ::libc::SPLICE_F_NONBLOCK;

    // From asm-generic/posix_types.h
    #[allow(non_camel_case_types)]
    pub type loff_t = c_longlong;

    extern {
        pub fn splice(fd_in: RawFd, off_in: *mut loff_t, fd_out: RawFd, off_out: *mut loff_t,
                      len: size_t, flags: c_uint) -> ssize_t;
    }
}

enum SpliceMode {
    Block,
    #[allow(dead_code)]
    NonBlock
}

// TODO: Replace most &RawFd with AsRawFd
fn splice(fd_in: &RawFd, fd_out: &RawFd, len: size_t, mode: SpliceMode) -> io::Result<ssize_t> {
    let flags = match mode {
        SpliceMode::Block => 0,
        SpliceMode::NonBlock => raw::SPLICE_F_NONBLOCK,
    };
    match unsafe { raw::splice(*fd_in, ptr::null_mut(), *fd_out, ptr::null_mut(), len, flags) } {
        -1 => Err(io::Error::last_os_error()),
        s => Ok(s),
    }
}

static SPLICE_BUFFER_SIZE: size_t = 1024;

/// Loop while reading and writing from one file descriptor to another using `splice(2)`.
/// The loop stop when `do_flush` is set to `true`.
/// At the end, a flush event is send to `flush_event` if any.
///
/// This function should be used in a dedicated thread, e.g. `thread::spawn(move ||
/// splice_loop(do_flush, None, rx.as_raw_fd(), tx.as_raw_fd()))`.
///
/// You should ensure that there is no append flag to the `fd_out` file descriptor.
/// You can use `unset_append_flag()` if needed and `set_flags()` to restore to the initial state.
pub fn splice_loop(do_flush: Arc<AtomicBool>, flush_event: Option<Sender<()>>, fd_in: RawFd, fd_out: RawFd) {
    'select: loop {
        if do_flush.load(Relaxed) {
            break 'select;
        }
        // FIXME: Add a select(2) watching for stdin and a pipe to stop the task
        // Need pipe to block on (the kernel only look at input)
        match splice(&fd_in, &fd_out, SPLICE_BUFFER_SIZE, SpliceMode::Block) {
            Ok(..) => {},
            Err(e) => {
                match e.kind() {
                    io::ErrorKind::BrokenPipe => {},
                    _ => {
                        do_flush.store(true, Relaxed);
                        break 'select;
                    }
                }
            }
        }
    }
    match flush_event {
        Some(event) => {
            let _ = event.send(());
        },
        None => {}
    }
}
