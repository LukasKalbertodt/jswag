use job::{Job, JobType};
use term_painter::{Color, ToStyle};
use java;


pub fn handle(job: Job) {
    for sj in &job.sub_jobs {
        match *sj {
            JobType::Check => {
                verbose! {{
                    println!(
                        "{} parsing & checking files [--check]",
                        Color::Green.bold().paint("> Starting action:")
                    );
                }}
            },
            JobType::PassThrough => {
                verbose! {{
                    println!(
                        "{} passing files to `javac` [--pass-through]",
                        Color::Green.bold().paint("> Starting action:")
                    );
                }}

                if java::compile_all(&job.files).is_err() {
                    note!("run `jswag` again with `--verbose` to obtain additional \
                        information.");
                }
            },
            JobType::Run => {
                verbose! {{
                    println!(
                        "{} running [--run]",
                        Color::Green.bold().paint("> Starting action:")
                    );
                }}

                if java::run_first_main(&job.files).is_err() {
                    note!("run `jswag` again with `--verbose` to obtain additional \
                        information.");
                }
            }
            ref sj => {
                println!("Ignoring sub job '{:?}'...", sj);
            }
        }
    }
}
