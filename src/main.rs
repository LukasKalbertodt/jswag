#![allow(unused_imports)]
extern crate term_painter;

use std::path::Path;
use std::error::Error;
use filemap::open_file;
use diagnostics::ErrorHandler;
use term_painter::{Color, ToStyle};
use syntax::parse::Parser;
use std::default::Default;

mod syntax;
mod diagnostics;
mod filemap;
mod style;


fn main() {
    let filemap = match open_file(Path::new("examples/Quersumme.java")) {
        Err(e) => panic!("Error opening file: {}", e.description()),
        Ok(fmap) => fmap,
    };

    let error_handler = ErrorHandler::new(filemap.clone());
    let style_config = style::Config::default();

    let toks = Box::new(syntax::Tokenizer::new(&filemap, &error_handler));
    let mut parser = Parser::new(toks, &error_handler);
    let checker = style::Checker::new(&error_handler, &style_config);


    match parser.parse_cunit() {
        Ok(cu) => checker.check(&cu),
        _ => {}
    }
    // println!("{:?}", cu.ok());


    // let reals = toks.filter(|t| t.tok.is_real());

    // // let mut t = term::stdout().unwrap();

    // // let mut old_line = 0;
    // for tok in reals {
    //     // printing line prefix
    //     // let new_line = filemap.get_loc(tok.span.lo).line;
    //     // if new_line > old_line {
    //     //     for i in old_line .. new_line {
    //     //         println!("");
    //     //         colored!(t, BLUE, (print!("{:>2}: ", i + 1)));
    //     //     }
    //     //     old_line = new_line;
    //     // }

    //     print!("{:?}{}", tok.tok, Color::Blue.paint("|"));
    //     // colored!(t, BLUE, print!("|"));

    // }
    // println!("");
}
