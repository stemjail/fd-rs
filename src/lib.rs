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

#![feature(into_raw_os)]
#![feature(libc)]

extern crate libc;

use std::io;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

mod pipe;
mod splice;

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
