use job::{Job, JobType};
use java;
use check;
use analyze;


pub fn handle(job: Job) -> Result<(), ()> {
    let mut check_res = None;
    for sj in &job.sub_jobs {
        match *sj {
            JobType::Check => {
                if job.verbose {
                    msg!(
                        Debug,
                        "Starting batch checking of {} file(s) [--check]",
                        job.files.len()
                    );
                }
                let res = check::check_all(&job);
                match res {
                    Err(_) => return Err(()),
                    Ok(res) => check_res = Some(res),
                }
            },
            JobType::PassThrough => {
                if job.verbose {
                    msg!(
                        Debug,
                        "Starting batch compile of {} file(s) [--pass-through]",
                        job.files.len()
                    );
                }

                if java::compile_all(&job).is_err() {
                    msg!(Aborting, "due to previous errors");
                    msg!(None, "run `jswag` again with `--verbose` or `-v` to \
                        obtain additional information.");
                    return Err(());
                }
            },
            JobType::Run => {
                if job.verbose {
                    msg!(
                        Debug,
                        "Starting to run one of {} file(s) [--run]",
                        job.files.len()
                    );
                }

                if java::run_first_main(&job).is_err() {
                    msg!(Aborting, "due to previous errors");
                    msg!(None, "run `jswag` again with `--verbose` or `-v` to \
                        obtain additional information.");
                    return Err(());
                }
            },
            JobType::Analyze { .. } => {
                if let Some(ref check_res) = check_res {
                    for &(ref file, ref ast) in check_res {
                        analyze::check_names(ast, file);
                    }
                } else {
                    msg!(Error, "All analyses required the 'check' job!");
                    return Err(());
                }
            },
            // ref sj => {
            //     msg!(Ignoring, "job '{:?}'", sj);
            // }
        }
    }

    Ok(())
}
