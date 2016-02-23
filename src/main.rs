extern crate docopt;
extern crate rustc_serialize;
extern crate xswag_base as base;
extern crate xswag_syntax_java as syntax;
extern crate term_painter;

use docopt::Docopt;
use term_painter::{Attr, Color, ToStyle};

#[macro_use]
mod ui;

mod args;
mod config;
mod dispatch;
mod java;
mod job;

use job::Job;

fn main() {
    use args::Args;

    // If there are no command line parameters, we print a nice message without
    // telling docopt.
    if std::env::args().count() == 1 {
        println!(
            "{} Run `{}` to show the usage message or `{}` to show the \
                version of jswag",
            Color::Red.bold().paint("No arguments given!"),
            Attr::Bold.paint("jswag --help"),
            Attr::Bold.paint("jswag (-V | --version)"),
        );
        std::process::exit(config::EXIT_NO_INPUT);
    }

    // Parse command line arguments with docopt and exit if anything went
    // wrong.
    let args: Args = Docopt::new(args::USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| {
            if e.fatal() {
                println!("{}", e);
                std::process::exit(config::EXIT_INVALID_INPUT);
            } else {
                println!("{}", e);
                std::process::exit(0)
            }
        });


    // If the `--version` flag was set, we do nothing but print the version.
    if args.flag_version {
        println!("jswag v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    // Check validity of args and compose them into a job object
    let job = match Job::from_args(args) {
        None => {
            println!("Abort due to CLI parameter errors...");
            std::process::exit(config::EXIT_INVALID_INPUT);
        },
        Some(j) => j,
    };

    // execute the job
    dispatch::handle(job);
}
