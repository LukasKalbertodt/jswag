pub const USAGE: &'static str = "
Usage: jswag build [options] <file>...
       jswag run [options] <file>...
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
pub struct Args {
    pub cmd_build: bool,
    pub cmd_run: bool,
    pub arg_file: Vec<String>,
    pub arg_analyze: Vec<String>,
    pub flag_check: bool,
    pub flag_pass_through: bool,
    pub flag_run: bool,
    pub flag_verbose: bool,
    pub flag_version: bool,
}

// TODO: add this
//      jswag [options] [<file>...]
// semantics are still quite unclear
