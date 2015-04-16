use std::iter::{Iterator};
use super::token::Token;
use super::lexer::Tokenizer;
use std::boxed::Box;
use super::ast;



pub struct Parser<'a>{
    // Token stream
    tstr: Box<Tokenizer<'a>>,

    // current token
    curr: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Box<Tokenizer<'a>>) -> Parser<'a> {
        Parser {
            tstr: lex,
            curr: None,
        }
    }

    // Parse a complete compilation unit
    pub fn parse_cunit(&mut self) -> ast::CUnit {
        ast::CUnit {
            imports: Vec::new(),
            class: ast::Class,
        }
    }



}
