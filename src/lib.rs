// Copyright (C) 2015 Mickaël Salaün
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

#![cfg(unix)]

extern crate libc;

use libc::{F_GETFL, F_SETFL, O_APPEND, c_int, fcntl};
use std::io;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

mod pipe;
mod splice;

#[cfg(not(target_os = "macos"))]
pub use splice::splice_loop;
pub use pipe::Pipe;

#[derive(Debug)]
/// Wrapper around a raw file descriptor.
pub struct FileDesc {
    fd: RawFd,
    close_on_drop: bool,
}

impl FileDesc {
    /// Set `close_on_drop` to `true` to close the inner file descriptor when the `FileDesc` is
    /// drop.
    pub fn new(fd: RawFd, close_on_drop: bool) -> FileDesc {
        FileDesc {
            fd: fd,
            close_on_drop: close_on_drop,
        }
    }

    /// Duplicate the inner file descriptor.
    pub fn dup(&self) -> io::Result<FileDesc> {
        Ok(FileDesc {
            fd: match unsafe { ::libc::dup(self.fd) } {
                -1 => return Err(io::Error::last_os_error()),
                n => n,
            },
            close_on_drop: self.close_on_drop,
        })
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        if self.close_on_drop {
            unsafe { ::libc::close(self.fd); }
        }
    }
}

impl AsRawFd for FileDesc {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(mut self) -> RawFd {
        self.close_on_drop = false;
        self.fd
    }
}

/// Return the original `fd` status flags if modified (cf. fcntl(2)).
pub fn unset_append_flag(fd: RawFd) -> io::Result<Option<c_int>> {
    let status = unsafe { fcntl(fd, F_GETFL) };
    if status == -1 {
        return Err(io::Error::last_os_error());
    }
    if status & O_APPEND == 0 {
        return Ok(None);
    }
    set_flags(fd, status & !O_APPEND)?;
    Ok(Some(status))
}

/// Set file status flags (cf. fcntl(2)).
pub fn set_flags(fd: RawFd, status: c_int) -> io::Result<()> {
    let ret = unsafe { fcntl(fd, F_SETFL, status) };
    if ret == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
