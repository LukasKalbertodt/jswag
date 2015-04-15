extern crate term_painter;


use std::path::Path;
use std::error::Error;
use filemap::open_file;
use diagnostics::ErrorHandler;
use term_painter::{Color, ToStyle};

mod syntax;
mod diagnostics;
mod filemap;


fn main() {
    let filemap = match open_file(Path::new("Quersumme.java")) {
        Err(e) => panic!("Error opening file: {}", e.description()),
        Ok(fmap) => fmap,
    };

    let error_handler = ErrorHandler::new(filemap.clone());

    let toks = syntax::Tokenizer::new(&filemap, &error_handler);
    let reals = toks.filter(|t| t.tok.is_real());

    // let mut t = term::stdout().unwrap();

    // let mut old_line = 0;
    for tok in reals {
        // printing line prefix
        // let new_line = filemap.get_loc(tok.span.lo).line;
        // if new_line > old_line {
        //     for i in old_line .. new_line {
        //         println!("");
        //         colored!(t, BLUE, (print!("{:>2}: ", i + 1)));
        //     }
        //     old_line = new_line;
        // }

        print!("{:?}{}", tok.tok, Color::Blue.paint("|"));
        // colored!(t, BLUE, print!("|"));

    }
    println!("");
}
