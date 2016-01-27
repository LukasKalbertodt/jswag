use std::process::{Command, ExitStatus};
use std::io;


/// Calls `javac` with the given files
pub fn compile(files: &[String]) -> Result<(), Error> {
    let javac_command = "javac";

    for file in files {
        // Print what we are about to do
        verbose! {{
            executing!("`{} {}`", javac_command, file);
        }}

        // Spawn new child process
        let child = Command::new(javac_command)
                            .arg(file)
                            .spawn();
        let mut child = try!(child);

        let status = try!(child.wait());

        // Stop processing, if javac failed to compile.
        if !status.success() {
            return Err(Error::JavacFailure(status));
        }
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
