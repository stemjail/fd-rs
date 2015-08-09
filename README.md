# fd-rs

*fd* is a library to handle file descriptors:
* `FileDesc` is a raw file descriptor wrapper how can properly close itself when dropped.
* `Pipe` is an interface to `pipe(2)`.
* The `loop_splice()` function can be used for zero-copy transfers using `splice(2)` (Linux specific).

This library is a work in progress.
The API may change.
