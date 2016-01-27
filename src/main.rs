extern crate docopt;
extern crate rustc_serialize;
extern crate xswag_base as base;
extern crate xswag_syntax_java as syntax;
extern crate term_painter;

use docopt::Docopt;

const USAGE: &'static str = "
Usage: jswag build [options] [<file>...]
       jswag watch [options] [<file>...]
       jswag [options] <file>...
       jswag (--help | --version)

Options:
    -a <check>, --analyze <check>   Run the given check.
    -c, --check                     Check files for language errors with
                                    internal tools.
    -h, --help                      Show this message.
    -p, --pass-through              Call `javac` to compile the files.
    -r, --run                       Tries to execute the compiled classes in
                                    the order they were given. Requires `-p`.
    -v, --verbose                   More verbose messages.
    -V, --version                   Show the version of jswag.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_build: bool,
    cmd_watch: bool,
    arg_file: Option<Vec<String>>,
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
        use term_painter::{Attr, Color, ToStyle};

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
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());


    // If the `--version` flag was set, we do nothing but print the version.
    if args.flag_version {
        println!("jswag v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    println!("{:#?}", args);
}
