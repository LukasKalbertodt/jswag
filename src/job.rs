use args::{Args, Encoding};
use std::collections::VecDeque;
use std::io;
use std::fs;
use std::path::PathBuf;

/// A job description to be executed.
///
/// Upon start of `jswag`, command line parameters are parsed into a `Job`
/// object, which is then handled. This object is then passed to many functions
/// further down the stack to be available in many places.
#[derive(Clone, Debug)]
pub struct Job {
    // TODO: maybe we should use a normal `Vec`
    pub sub_jobs: VecDeque<JobType>,
    pub files: Vec<PathBuf>,
    pub verbose: bool,
    pub lossy_decoding: bool,
    pub encoding: Encoding,
}

impl Job {
    pub fn from_args(mut args: Args) -> Option<Self> {
        let no_cmd = !(args.cmd_build || args.cmd_run || args.cmd_raw);
        if !no_cmd && args.arg_file.is_empty() {
            args.arg_file.push(".".into());
        }
        let files = match Self::fold_files(args.arg_file) {
            Err(e) => {
                msg!(Error, "An IO error occured while analysing file list: {}", e);
                return None;
            },
            Ok(ref f) if f.is_empty() => {
                msg!(Error, "Filelist is empty!");
                return None;
            }
            Ok(f) => f,
        };

        let mut out = Job {
            sub_jobs: VecDeque::new(),
            files: files,
            verbose: args.flag_verbose,
            lossy_decoding: args.flag_lossy_decoding,
            encoding: args.flag_encoding,
            // encoding: Encoding::Utf8,
        };

        // Matching flag, implying flags or implying commands
        if args.flag_check || !args.arg_analyze.is_empty() ||
            args.cmd_run || args.cmd_build || no_cmd
        {
            out.sub_jobs.push_back(JobType::Check);
        }
        // Matching argument or implying commands
        if !args.arg_analyze.is_empty() || args.cmd_run || args.cmd_build || no_cmd {
            if args.cmd_run || args.cmd_build {
                args.arg_analyze.push("style".into());
            }

            let passes: VecDeque<_> = args.arg_analyze.iter().filter_map(|name| {
                match &name[..] {
                    "style" => Some(AnalyzePass::Style),
                    _ => {
                        println!("Invalid analysis pass '{}'", name);
                        None
                    }
                }
            }).collect();

            // There was at least one invalid pass name
            if passes.len() != args.arg_analyze.len() {
                return None;
            }

            out.sub_jobs.push_back(
                JobType::Analyze {
                    passes: passes,
                }
            );
        }

        if args.flag_pass_through || args.cmd_run || args.cmd_build {
            out.sub_jobs.push_back(JobType::PassThrough);
        }
        if args.flag_run || args.cmd_run {
            if out.sub_jobs.iter().find(|&sj| sj == &JobType::PassThrough).is_none() {
                println!("In order to `--run`, `--pass-through` needs to be set");
                return None;
            }
            out.sub_jobs.push_back(JobType::Run);
        }

        Some(out)
    }

    pub fn fold_files(files: Vec<String>) -> io::Result<Vec<PathBuf>> {
        let mut new = Vec::new();
        for file in files {
            let p = PathBuf::from(file);
            let meta = try!(fs::metadata(&p));

            if meta.is_file() {
                new.push(p);
            } else if meta.is_dir() {
                let entries = try!(fs::read_dir(p))
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                    .map(|f| f.path())
                    .filter(|p| {
                        p.extension()
                            .map(|ext| ext == "java" || ext == "jav")
                            .unwrap_or(false)
                    });
                new.extend(entries);
            }
        }
        Ok(new)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobType {
    /// Checks input files for language errors with internal tools
    Check,
    ///
    Analyze {
        passes: VecDeque<AnalyzePass>,
    },
    /// Forwards files to `javac` to compile them into byte code
    PassThrough,
    /// Runs `java` to execute the files
    Run,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalyzePass {
    Style,
}
