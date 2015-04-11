extern crate term;


use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::io::Read;


mod syntax;


fn main() {
    let path = Path::new("Quersumme.java");
    let mut file = match File::open(path) {
        Err(e) => panic!("Could not open '{}': {}", path.display(),
            e.description()),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `IoResult<String>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(e) => panic!("Could not read '{}': {}", path.display(),
            e.description()),
        // Ok(_) => println!("'{}' contains:\n{}", path.display(), s),
        Ok(_) => {},
    }

    macro_rules! colored {
        ($t:ident, $c:ident, $p:expr ) => ({
            $t.fg(term::color::$c).unwrap();
            $p;
            $t.reset().unwrap();
        })
    }


    let toks = syntax::Tokenizer::new(&s);
    let reals = toks.filter(|t| t.tok.is_real());

    let mut t = term::stdout().unwrap();

    let mut c = 1;
    colored!(t, BLUE, (print!("{:>2}: ", c)));
    for tok in reals {
        if tok.tok == syntax::Token::Whitespace(true) {
            println!("");
            c += 1;
            colored!(t, BLUE, (print!("{:>2}: ", c)));
        } else {
            print!("{:?}", tok.tok);
            colored!(t, YELLOW, print!("_"));
        }
    }
    // let tok = tokenizer.next().unwrap();
    // loop {
    //     match tok.ty {
    //         syntax::TokenType::Other(ref s) => println!("{}", s),
    //         _ => {},
    //     }

    //     let tok = match tokenizer.next() {
    //         None => break,
    //         Some(t) => t,
    //     };
    // }
    println!("");
}
