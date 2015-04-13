extern crate term;


use std::path::Path;
use std::error::Error;
use filemap::open_file;
use diagnostics::ErrorHandler;


mod syntax;
mod diagnostics;
mod filemap;


macro_rules! colored {
    ($t:ident, $c:ident, $p:expr ) => ({
        $t.fg(term::color::$c).unwrap();
        $p;
        $t.reset().unwrap();
    })
}

fn main() {
    let filemap = match open_file(Path::new("Quersumme.java")) {
        Err(e) => panic!("Error opening file: {}", e.description()),
        Ok(fmap) => fmap,
    };

    let error_handler = ErrorHandler::new(filemap.clone());

    let toks = syntax::Tokenizer::new(&filemap, &error_handler);
    let reals = toks.filter(|t| t.tok.is_real());

    let mut t = term::stdout().unwrap();

    let mut old_line = 0;
    for tok in reals {
        // printing line prefix
        if tok.line > old_line {
            for i in old_line .. tok.line {
                println!("");
                colored!(t, BLUE, (print!("{:>2}: ", i + 1)));
            }
            old_line = tok.line;
        }

        print!("{:?}", tok.tok);
        colored!(t, GREEN, print!("["));
        // print!("{:?}", filemap.get_loc(tok.span.0));
        colored!(t, GREEN, print!("]"));
        colored!(t, YELLOW, print!("_"));

    }
    println!("");
}
