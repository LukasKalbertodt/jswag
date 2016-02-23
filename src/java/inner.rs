use std::process::{Command, ExitStatus};
use std::path::Path;
use std::io;
use super::{JAVAC_NAME, JAVA_NAME};


/// Calls `javac` with the given files
pub fn compile(file: &str) -> Result<(), Error> {
    // Print what we are about to do
    msg!(Running, "`{} {}`", JAVAC_NAME, file);

    // Spawn new child process
    let child = Command::new(JAVAC_NAME)
                        .arg(file)
                        .spawn();
    let mut child = try!(child);

    let status = try!(child.wait());

    // Stop processing, if javac failed to compile.
    if !status.success() {
        return Err(Error::JavacFailure(status));
    }

    Ok(())
}

pub fn run<P: AsRef<Path>>(class: &str, path: P) -> Result<(), Error> {
    // Print what we are about to do
    msg!(Running, "`{} {}`", JAVA_NAME, class);

    // Spawn new child process
    let child = Command::new(JAVA_NAME)
                        .arg(class)
                        .current_dir(path)
                        .spawn();
    let mut child = try!(child);

    let status = try!(child.wait());

    // Stop processing, if javac failed to compile.
    if !status.success() {
        return Err(Error::JavacFailure(status));
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    JavaBinaryNotFound,
    JavacFailure(ExitStatus),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::NotFound {
            Error::JavaBinaryNotFound
        } else {
            Error::Io(e)
        }
    }
}
