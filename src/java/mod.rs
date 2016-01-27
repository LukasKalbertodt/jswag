/// Functionality to call original `java` and `javac` commands.
///
/// The actual functionality is implemented in submodule `inner`. This parent
/// module contains functions for pretty printing the output of `inner`.

mod inner;

use std::path::Path;
use self::inner::Error;

const JAVAC_NAME: &'static str = "javac";
const JAVA_NAME: &'static str = "java";

/// Calls `javac` with the given files
pub fn compile_all(files: &[String]) -> Result<(), ()> {
    for file in files {
        try!(compile(file));
    }
    Ok(())
}
pub fn compile(file: &str) -> Result<(), ()> {

    inner::compile(file).map_err(|e| {
        match e {
            Error::JavaBinaryNotFound => {
                error!(
                    "`{1}` was not found on the system. Make sure that `{1}` \
                        is installed and in your PATH. Aborting.",
                    JAVAC_NAME
                );
            },
            Error::JavacFailure(status) => {
                error!(
                    "`{}` exited with a non-success status ({}). Aborting.",
                    JAVAC_NAME,
                    status
                );
            },
            Error::Io(e) => {
                error!(
                    "an IO error occured while executing `{}`: {}. Aborting.",
                    JAVAC_NAME,
                    e
                );
            }
        };
    })
}

pub fn run_first_main(files: &[String]) -> Result<(), ()> {
    // TODO: find the first class where `main` is defined
    let p = Path::new(&files[0]);
    let class = p.file_name().and_then(|s| s.to_str()).unwrap();
    let class = strip_file_ending(class);
    let parent = p.parent().unwrap();

    inner::run(class, parent).map_err(|e| {
        match e {
            Error::JavaBinaryNotFound => {
                error!(
                    "`{1}` was not found on the system. Make sure that `{1}` \
                        is installed and in your PATH. Aborting.",
                    JAVA_NAME
                );
            },
            Error::JavacFailure(status) => {
                error!(
                    "`{}` exited with a non-success status ({}). Aborting.",
                    JAVA_NAME,
                    status
                );
            },
            Error::Io(e) => {
                error!(
                    "an IO error occured while executing `{}`: {}. Aborting.",
                    JAVA_NAME,
                    e
                );
            }
        };
    })
}

fn strip_file_ending(file: &str) -> &str {
    if file.ends_with(".java") {
        &file[0..file.len() - 5]
    } else if file.ends_with(".jav") {
        &file[0..file.len() - 4]
    } else {
        file
    }
}
