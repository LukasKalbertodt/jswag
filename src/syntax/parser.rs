use std::iter::{Iterator};
use super::token::*;
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

// macro_rules! expect {
//     (
//         $self_:expr;
//         $( $pattern:expr => $arm:expr),*
//     ) => (
//         match $self_.curr() {
//             $( $pattern => $arm , )*
//             a @ _ => $self_.err_unexpected( $( $pattern ,)*, a),
//         }

//     )
// }


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
        loop {
            if self.curr.is_none() {
                break;
            }

            let curr = try!(self.curr());
            match curr.tok {
                Token::Keyword(Keyword::Import) => {
                    self.bump();
                    cu.imports.push(try!(self.parse_import()));
                },
                _ => break,
            }
        }

        Ok(cu)
    }

    fn parse_import(&mut self) -> PResult<ast::Import> {
        // Token `Import` is already eaten.
        let mut name = ast::Name { path: Vec::new(), last: None };
        let mut w = try!(self.eat_word());
        loop {
            match try!(self.curr()).tok {
                Token::Semi => {
                    name.last = Some(w);
                    self.bump();
                    return Ok(ast::Import::Single(name));
                },
                Token::Dot => {
                    name.path.push(w);
                    self.bump();
                },
                f @ _ => return Err(self.err_unexpected(
                    &[Token::Semi, Token::Dot],f)),
            }

            match try!(self.curr()).tok {
                Token::Star => {
                    self.bump();
                    try!(self.eat(Token::Semi));
                    return Ok(ast::Import::Wildcard(name));
                },
                Token::Word(s) => {
                    self.bump();
                    w = s;
                },
                f @ _ => return Err(self.err_unexpected(
                    &[Token::Star, Token::Word("".to_string())], f)),
            }
        }
    }

    fn eat_word(&mut self) -> PResult<String> {
        let curr = try!(self.curr());

        match curr.tok {
            Token::Word(s) => {
                self.bump();
                Ok(s)
            },
            _ => Err(self.err_unexpected(
                &[Token::Word("".to_string())], curr.clone().tok)),
        }
    }

    fn eat(&mut self, t: Token) -> PResult<()> {
        let curr = try!(self.curr());
        if curr.tok == t {
            self.bump();
            Ok(())
        } else {
            Err(self.err_unexpected(&[t], curr.tok))
        }
    }

    fn eat_maybe(&mut self, t: Token) -> PResult<bool> {
        let curr = try!(self.curr());
        if curr.tok == t {
            self.bump();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn expect_one_of(&mut self, eat: &[Token]/*, spare: &[Token]*/)
        -> PResult<TokenSpan> {
        let curr = try!(self.curr());
        if eat.contains(&curr.tok) {
            self.bump();
            Ok(curr)
        } else {
            Err(self.err_unexpected(eat, curr.tok))
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

    fn curr(&mut self) -> PResult<TokenSpan> {
        match self.curr {
            None => Err(self.err_eof()),
            Some(ref curr) => Ok(curr.clone()),
        }
    }

    // Error reporting stuff
    fn err_eof(&self) -> PErr {
        self.e.err("Expected token, found '<eof'>!");
        PErr::Fatal
    }

    fn err_unexpected(&self, expected: &[Token], found: Token) -> PErr {
        let list = expected.iter().enumerate()
            .fold(String::new(), |mut list, (idx, ref t)| {
            list.push_str(&*format!("`{}`", t.as_java_string()));
            if idx < expected.len() - 2 {
                list.push_str(", ");
            } else if idx == expected.len() - 2 {
                list.push_str(" or ");
            }
            list
        });

        self.e.span_err(self.curr.clone().unwrap().span,
            format!("Unexpected token: Expected {}, found `{}`",
                list, found.as_java_string()).as_ref());
        PErr::Fatal
    }
}
