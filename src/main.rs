extern crate docopt;
extern crate rustc_serialize;
extern crate xswag_base as base;
extern crate xswag_syntax_java as syntax;
extern crate term_painter;
#[macro_use]
extern crate lazy_static;

use docopt::Docopt;
use std::sync::atomic::{AtomicBool, Ordering};
use term_painter::{Attr, Color, ToStyle};

#[macro_use]
mod util;
mod args;
mod java;
mod job;

use job::Job;


/// We store globally if the `--verbose` flag was set. This might change later
/// on, since it doesn't scale well and is ugly. Easy for now, though.
lazy_static! {
    static ref VERBOSE: AtomicBool = AtomicBool::new(false);
}

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
        return;
    }

    // Parse command line arguments with docopt and exit if anything went
    // wrong.
    let args: Args = Docopt::new(args::USAGE)
                                .and_then(|d| d.decode())
                                .unwrap_or_else(|e| e.exit());


    // If the `--version` flag was set, we do nothing but print the version.
    if args.flag_version {
        println!("jswag v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let job = Job::from_args(args);

    // // ----- Now we have to decide what actions to execute -------------------
    // // TODO: filter/check file list
    // // First main action: parse Java file ourselves (`--check`)
    // if args.flag_check || !args.arg_analyze.is_empty() {
    //     verbose! {{
    //         println!(
    //             "{} parsing & checking files [--check]",
    //             Color::Green.bold().paint("> Starting action:")
    //         );
    //     }}
    // }

    // // Second main action: pass through to `javac` (`--pass-through`)
    // if args.flag_pass_through {
    //     verbose! {{
    //         println!(
    //             "{} passing files to `javac` [--pass-through]",
    //             Color::Green.bold().paint("> Starting action:")
    //         );
    //     }}

    //     if java::compile_all(&args.arg_file).is_err() {
    //         note!("run `jswag` again with `--verbose` to obtain additional \
    //             information.");
    //     }
    // }

    // // Third main action: run compiled files (`--run`)
    // if args.flag_run {
    //     verbose! {{
    //         println!(
    //             "{} running [--run]",
    //             Color::Green.bold().paint("> Starting action:")
    //         );
    //     }}

    //     if java::run_first_main(&args.arg_file).is_err() {
    //         note!("run `jswag` again with `--verbose` to obtain additional \
    //             information.");
    //     }
    // }
}
