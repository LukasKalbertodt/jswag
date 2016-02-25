use job::{Job, JobType};
use java;
use check;


pub fn handle(job: Job) -> Result<(), ()> {
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
                if res.is_err() {
                    return Err(());
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
            }
            ref sj => {
                msg!(Ignoring, "job '{:?}'", sj);
            }
        }
    }

    Ok(())
}
