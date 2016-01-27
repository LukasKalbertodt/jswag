/// Functionality to call original `java` and `javac` commands.
///
/// The actual functionality is implemented in submodule `inner`. This parent
/// module contains functions for pretty printing the output of `inner`.

use std::convert::From;
use std::io;

mod inner;

/// Calls `javac` with the given files
pub fn compile(files: &[String]) -> Result<(), ()> {
    use self::inner::Error;

    inner::compile(files).map_err(|e| {
        match e {
            Error::JavaBinaryNotFound => {
                error!("`javac` was not found on the system. Make sure that \
                        `javac` is installed and in your PATH. Aborting.");
            },
            Error::JavacFailure(status) => {
                error!(
                    "`javac` exited with a non-success status ({}). Aborting.",
                    status
                );
            },
            Error::Io(e) => {
                error!(
                    "an IO error occured while executing `javac`: {}. Aborting.",
                    e
                );
            }
        };
    })
}
