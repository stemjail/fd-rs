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

use std::fs::File;
use std::io;
use std::os::unix::io::FromRawFd;

mod raw {
    use libc::c_int;

    // From asm-generic/fcntl.h
    pub const O_CLOEXEC: c_int = 0o2000000;

    #[repr(C)]
    pub struct PipeFds {
        pub reader: c_int,
        pub writer: c_int,
    }

    extern {
        pub fn pipe2(fds: *mut PipeFds, flags: c_int) -> c_int;
    }
}

/// A thread-safe `pipe(2)` interface.
///
/// Create a reader and a writer `File` for each part of the pipe.
pub struct Pipe {
    pub reader: File,
    pub writer: File,
}

impl Pipe {
    pub fn new() -> io::Result<Pipe> {
        let mut fds = raw::PipeFds {
            reader: -1,
            writer: -1
        };
        if unsafe { raw::pipe2(&mut fds, raw::O_CLOEXEC) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Pipe {
            reader: unsafe { File::from_raw_fd(fds.reader) },
            writer: unsafe { File::from_raw_fd(fds.writer) },
        })
    }
}
