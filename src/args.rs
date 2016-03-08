pub const USAGE: &'static str = "
Usage: jswag build [options] [<file>...]
       jswag run [options] [<file>...]
       jswag [options] <file>...
       jswag raw [<file>...]
       jswag (--help | --version)

Commands:
    build       Compiles all files and runs simple analysis checks on them.
                Automatically adds these parameters:
                    $ --check --pass-through --analyze style
    run         Works like `build`, but runs the file afterwards. Automatically
                adds these parameters to the already added parameters of
                `build`:
                    $ --run
    <none>      For compatibility this works similar to the original `javac`
                command. Right now it's exactly the same as 'build', except
                that the file list musn't be empty.
    raw         Does nothing automatically. Every task has to be explicitly
                stated with command line parameters.

Actions:
    -a <check>, --analyze <check>   Run the given check. Implies `-c`.
    -c, --check                     Check files for language errors with
                                    internal tools.
    -p, --pass-through              Call `javac` to compile the files.
    -r, --run                       Tries to execute the compiled classes in
                                    the order they were given. Requires `-p`.

Options:
    --lossy-decoding        Replace invalid UTF-8 or UTF-16 characters in the
                            input file with 'U+FFFD REPLACEMENT CHARACTER' (�)
                            instead of exiting.
    --encoding <encoding>   Forces a specific file decoding. Valid
                            values: 'utf8' [default: utf8]
    -h, --help              Show this message.
    -v, --verbose           More verbose messages.
    -V, --version           Show the version of jswag.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    pub cmd_build: bool,
    pub cmd_run: bool,
    pub cmd_raw: bool,
    pub arg_file: Vec<String>,
    pub arg_analyze: Vec<String>,
    pub flag_encoding: Encoding,
    pub flag_check: bool,
    pub flag_pass_through: bool,
    pub flag_run: bool,
    pub flag_verbose: bool,
    pub flag_version: bool,
    pub flag_lossy_decoding: bool,
}

#[derive(Clone, Copy, RustcDecodable, Debug)]
pub enum Encoding {
    Utf8,
    // TODO: fucking encoding
    // Utf16,
}
