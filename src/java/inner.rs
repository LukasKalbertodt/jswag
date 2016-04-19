use std::process::{Command, ExitStatus};
use std::path::Path;
use std::io;
use super::{JAVAC_NAME, JAVA_NAME};
use job::Job;


/// Calls `javac` with the given file
pub fn compile(file: &Path, job: &Job) -> Result<(), Error> {
    // Print what we are about to do
    if job.verbose {
        msg!(Running, "`{} {}`", JAVAC_NAME, file.display());
    }

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

/// Calls `java` with the given file
pub fn run<P: AsRef<Path>>(class: &str, path: P, job: &Job)
    -> Result<(), Error>
{
    // Print what we are about to do
    if job.verbose {
        msg!(Running,
            "`{} {}` in working directory '{}'",
            JAVA_NAME,
            class,
            path.as_ref().display()
        );
    }

    // Spawn new child process
    let child = Command::new(JAVA_NAME)
                        .arg(class)
                        // .current_dir(path)
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
