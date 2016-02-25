use std::io::{self, Read};
use job::Job;
use std::fs::File;
use base::{code, diag};
use syntax;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    // CriticalReport(diag::Report),
    Unknown,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

pub fn check_all(job: &Job) -> Result<Vec<()>, ()> {
    for file in &job.files {
        msg!(Checking, "'{}'", file);

        let res = check_file(job, file);
        if let Err(e) = res {
            match e {
                Error::Io(e) => {
                    match e.kind() {
                        io::ErrorKind::NotFound => {
                            msg!(Error, "File not found: {}", file);
                        },
                        _ => msg!(Error, "IO error: {:?}", e),
                    }
                },
                _ => println!("{:?}", e),
            };
            return Err(());
        }
    }
    Ok(vec![])
}

fn check_file(job: &Job, file_name: &str) -> Result<(), Error> {
    let mut file = try!(File::open(file_name));
    let mut src = String::new();
    try!(file.read_to_string(&mut src));

    let file_map = code::FileMap::new(file_name, src);
    let (res, errors) = syntax::parse_compilation_unit(&file_map);

    if let Err(e) = res.as_ref() {
        diag::print(&e, &file_map, diag::PrintOptions::default());
    }

    for e in &errors {
        diag::print(&e, &file_map, diag::PrintOptions::default());
    }


    // res.map(|_| ()).map_err(|_| ())
    if res.is_err() || !errors.is_empty() {
        Err(Error::Unknown)
    } else {
        Ok(())
    }
}
