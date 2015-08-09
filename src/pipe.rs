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


use libc::c_int;
use std::fs::File;
use std::io;
use std::os::unix::io::FromRawFd;

/// A `pipe(2)` interface.
///
/// Create a reader and a writer `File` for each part of the pipe.
pub struct Pipe {
    pub reader: File,
    pub writer: File,
}

impl Pipe {
    pub fn new() -> io::Result<Pipe> {
        let mut fds: (c_int, c_int) = (-1, -1);
        let fdp: *mut c_int = unsafe { ::std::mem::transmute(&mut fds) };
        // TODO: Use pipe2(2) with O_CLOEXEC
        if unsafe { ::libc::pipe(fdp) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Pipe {
            reader: unsafe { File::from_raw_fd(fds.0) },
            writer: unsafe { File::from_raw_fd(fds.1) },
        })
    }
}
