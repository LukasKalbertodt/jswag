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

    // let test = r#"
// /******* Quersumme.java  *****/
// import AlgoTools.IO;
// // We need stuff
// public class Quersumme {
// }
//     "#.to_string();

    let toks = syntax::Tokenizer::new(&s);
    let reals = toks.filter(|t| t.tok.is_real());
    for tok in reals {
        if tok.tok == syntax::Token::Whitespace(true) {
            println!("");
        } else {
            print!("{:?}   ", tok.tok);
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
