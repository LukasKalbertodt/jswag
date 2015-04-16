use std::iter::{Iterator};
use super::token::{Token, Keyword};
use super::lexer::{Tokenizer, TokenSpan};
use std::boxed::Box;
use super::ast;
use filemap::Span;
use std::result::Result;
use diagnostics::ErrorHandler;
use std::mem::swap;

enum PErr {
    Fatal,
}

pub type PResult<T> = Result<T, PErr>;


pub struct Parser<'a>{
    // Token stream
    tstr: Box<Tokenizer<'a>>,
    e: &'a ErrorHandler,

    // current token
    curr: Option<TokenSpan>,
    peek: Option<TokenSpan>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Box<Tokenizer<'a>>, e: &'a ErrorHandler) -> Parser<'a> {
        let mut p = Parser {
            tstr: lex,
            e: e,
            curr: None,
            peek: None,
        };
        p.dbump();
        p
    }

    // Parse a complete compilation unit
    pub fn parse_cunit(&mut self) -> PResult<ast::CUnit> {
        let mut cu = ast::CUnit {
            imports: Vec::new(),
            class: ast::Class,
        };
        let exp = [
            Token::Keyword(Keyword::Import),
            Token::Keyword(Keyword::Class)];
        match try!(self.expect(&exp)).tok {
            Token::Keyword(Keyword::Import) => {
                cu.imports.push(try!(self.parse_import()));
            },
            Token::Keyword(Keyword::Class) => {
                // self.parse_class()
            },
            _ => unreachable!(),
        }


        Ok(cu)
    }

    fn parse_import(&mut self) -> PResult<ast::Import> {
        // token Import is already eaten
        loop {
            try!(self.expect_word());
            match try!(self.expect(&[Token::Dot, Token::Semi])).tok {
                Token::Semi => break,
                _ => {},
            }
        }
        Ok(ast::Import::SingleType)
    }

    fn expect_word(&mut self) -> PResult<&TokenSpan> {
        match self.curr {
            None => Err(self.err_eof()),
            Some(ref curr) => {
                match curr.tok {
                    Token::Word(..) => Ok(curr),
                    _ => Err(self.err_wrong(&[], curr.clone().tok)),
                }
            }
        }
    }

    fn expect(&mut self, eat: &[Token]/*, spare: &[Token]*/)
        -> PResult<TokenSpan> {
        match self.curr.clone() {
            None => {
                Err(self.err_eof())
            },
            Some(curr) => {
                if eat.contains(&curr.tok) {
                    self.bump();
                    Ok(curr)
                } else {
                    Err(self.err_wrong(eat, curr.tok))
                }
            }
        }
    }

    // Advances by one token
    fn bump(&mut self) {
        swap(&mut self.curr, &mut self.peek);
        self.peek = self.tstr.next_real();
    }

    fn dbump(&mut self) {
        self.bump();
        self.bump();
    }

    // Error reporting stuff
    fn err_eof(&self) -> PErr {
        self.e.err("Expected token, found '<eof'>!");
        PErr::Fatal
    }

    fn err_wrong(&self, expected: &[Token], found: Token) -> PErr {
        self.e.span_err(self.curr.clone().unwrap().span,
            format!("Unexpected token: Found {:?}", found).as_ref());
        PErr::Fatal
    }
}
