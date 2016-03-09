use std::io::{self, Read};
use job::Job;
use std::fs::File;
use base::{code, diag};
use syntax::{self, ast};
use std;
use args::Encoding;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8(std::str::Utf8Error),
    // CriticalReport(diag::Report),
    Unknown,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}
impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Error {
        Error::Utf8(e.utf8_error())
    }
}

pub fn check_all(job: &Job) -> Result<Vec<(code::FileMap, ast::CompilationUnit)>, ()> {
    let mut out = Vec::new();
    for file in &job.files {
        msg!(Checking, "'{}'", file.display());

        let res = check_file(job, file);
        if let Err(e) = res {
            match e {
                Error::Io(e) => {
                    match e.kind() {
                        io::ErrorKind::NotFound => {
                            msg!(Error, "File not found: {}", file.display());
                        },
                        _ => msg!(Error, "IO error: {:?}", e),
                    }
                },
                Error::Utf8(e) => {
                    msg!(Error, "File '{}' doesn't contain valid UTF-8 ({})", file.display(), e);
                    msg!(Note, "Convert your file into valid UTF-8 or use the \
                        flag `--lossy-decoding`")
                },
                Error::Unknown => {},
                // _ => println!("{:?}", e),
            };
            return Err(());
        } else {
            out.push(res.unwrap());
        }

    }
    Ok(out)
}

fn check_file(job: &Job, file_name: &Path)
    -> Result<(code::FileMap, ast::CompilationUnit), Error>
{
    // read file contents into buffer
    let mut file = try!(File::open(file_name));
    let mut buffer = Vec::new();
    try!(file.read_to_end(&mut buffer));

    // try to decode input stream as Unicode
    let src = match job.encoding {
        Encoding::Utf8 => {
            if job.lossy_decoding {
                String::from_utf8_lossy(&buffer).into_owned()
            } else {
                try!(String::from_utf8(buffer))
            }
        },
        // TODO: fucking encoding
        // Encoding::Utf16 => {
        //     // check endianess
        //     if buffer.len() < 2 {

        //     }
        //     let be_bom = match (buffer[0], buffer[1]) {
        //         (0xFE, 0xFF) => Some(true),
        //         (0xFF, 0xFE) => Some(false),
        //         _ => None,
        //     };

        //     if job.lossy_decoding {
        //         String::from_utf16_lossy(&buffer)
        //     } else {
        //         try!(String::from_utf16(&buffer))
        //     }
        // },
    };

    // create filemap and parse
    let lossy_filename = file_name.to_string_lossy().into_owned();
    let file_map = code::FileMap::new(lossy_filename, src);
    let (ast, errors) = syntax::parse_compilation_unit(&file_map);

    let mut critical = false;
    for e in &errors {
        diag::print(&e, &file_map, diag::PrintOptions::default());

        if e.kind == diag::ReportKind::Error {
            critical = true;
        }
    }

    match (ast, critical) {
        (None, _) | (_, true) => Err(Error::Unknown),
        (Some(ast), _) => Ok((file_map, ast)),
    }
}
