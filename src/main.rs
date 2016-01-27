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
mod java;

/// We store globally if the `--verbose` flag was set. This might change later
/// on, since it doesn't scale well and is ugly. Easy for now, though.
lazy_static! {
    static ref VERBOSE: AtomicBool = AtomicBool::new(false);
}

const USAGE: &'static str = "
Usage: jswag build [options] [<file>...]
       jswag run [options] [<file>...]
       jswag [options] <file>...
       jswag (--help | --version)

Commands:
    build       Compiles all files and runs simple analysis checks on them.
                Automatically adds these parameters:
                    $ --check --pass-through --analyze style
    run         Works like `build`, but runs the file afterwards. Automatically
                adds these parameters to the already added parameters of
                `build`:
                    $ --run

Actions:
    -a <check>, --analyze <check>   Run the given check. Implies `-c`.
    -c, --check                     Check files for language errors with
                                    internal tools.
    -p, --pass-through              Call `javac` to compile the files.
    -r, --run                       Tries to execute the compiled classes in
                                    the order they were given. Requires `-p`.

Options:
    -h, --help      Show this message.
    -v, --verbose   More verbose messages.
    -V, --version   Show the version of jswag.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_build: bool,
    cmd_run: bool,
    arg_file: Vec<String>,
    arg_analyze: Vec<String>,
    flag_check: bool,
    flag_pass_through: bool,
    flag_run: bool,
    flag_verbose: bool,
    flag_version: bool,
}

fn main() {
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
    let mut args: Args = Docopt::new(USAGE)
                                .and_then(|d| d.decode())
                                .unwrap_or_else(|e| e.exit());


    // If `--verbose` was set, we have to set our global variable
    if args.flag_verbose {
        VERBOSE.store(true, Ordering::SeqCst);
    }

    // If the `--version` flag was set, we do nothing but print the version.
    if args.flag_version {
        println!("jswag v{}", env!("CARGO_PKG_VERSION"));
        return;
    }


    // ----- Apply commands overrides ----------------------------------------
    if args.cmd_build || args.cmd_run {
        args.flag_check = true;
        args.flag_pass_through = true;
    }
    if args.cmd_run {
        args.flag_run = true;
    }


    // ----- Now we have to decide what actions to execute -------------------
    // TODO: filter/check file list
    // First main action: parse Java file ourselves (`--check`)
    if args.flag_check || !args.arg_analyze.is_empty() {
        verbose! {{
            println!(
                "{} parsing & checking files [--check]",
                Color::Green.bold().paint("> Starting action:")
            );
        }}
    }

    // Second main action: pass through to `javac` (`--pass-through`)
    if args.flag_pass_through {
        verbose! {{
            println!(
                "{} passing files to `javac` [--pass-through]",
                Color::Green.bold().paint("> Starting action:")
            );
        }}

        if java::compile_all(&args.arg_file).is_err() {
            note!("run `jswag` again with `--verbose` to obtain additional \
                information.");
        }
    }

    // Third main action: run compiled files (`--run`)
    if args.flag_run {
        verbose! {{
            println!(
                "{} running [--run]",
                Color::Green.bold().paint("> Starting action:")
            );
        }}

        if java::run_first_main(&args.arg_file).is_err() {
            note!("run `jswag` again with `--verbose` to obtain additional \
                information.");
        }
    }
}
