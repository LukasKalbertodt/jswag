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

    // Parse a complete compilation unit. A CU consists of:
    // [PackageDecls] [ImportDecls] [TypeDecls]
    // A TypeDecl is either a ClassDecl or InterfaceDecl
    pub fn parse_cunit(&mut self) -> PResult<ast::CUnit> {
        let mut cu = ast::CUnit {
            imports: Vec::new(),
            class: None,
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
                Token::Keyword(Keyword::Public)
                    | Token::Keyword(Keyword::Class) => {
                    cu.class = Some(try!(self.parse_top_lvl_class()));
                }
                _ => break,
            }
        }

        Ok(cu)
    }

    fn parse_top_lvl_class(&mut self) -> PResult<ast::TopLevelClass> {
        let mut c = ast::TopLevelClass {
            visibility: ast::Visibility::Package,
            name: "".to_string(),
        };

        // Parse class modifier, until `class` appears
        // TODO: More class modifier (static, ..)
        loop {
            let curr = try!(self.curr());
            match curr.tok {
                Token::Keyword(Keyword::Public) => {
                    c.visibility = ast::Visibility::Public;
                    self.bump();
                },
                Token::Keyword(Keyword::Class) => {
                    self.bump();
                    break;
                },
                o @ _ => {
                    let ex = &[Token::Keyword(Keyword::Public),
                        Token::Keyword(Keyword::Class)];
                    return Err(self.err_unexpected(ex, o));
                }
            }
        }

        // `class` was parsed, next token should be class name
        c.name = try!(self.eat_word());

        // TODO: Type Params
        // TODO: Super class
        // TODO: implements

        // try!(self.eat(Token::OpenDelim(DelimToken::Brace)));

        Ok(c)
    }

    fn parse_import(&mut self) -> PResult<ast::Import> {
        // `import` has already been eaten.
        let mut name = ast::Name { path: Vec::new(), last: None };

        // The first token after `import` needs to be a word.
        let mut w = try!(self.eat_word());

        loop {
            match try!(self.curr()).tok {
                // End of `import` -> Eat Semi and return name.
                Token::Semi => {
                    name.last = Some(w);
                    self.bump();
                    return Ok(ast::Import::Single(name));
                },
                // Name continues
                Token::Dot => {
                    name.path.push(w);
                    self.bump();
                },
                f @ _ => return Err(self.err_unexpected(
                    &[Token::Semi, Token::Dot],f)),
            }

            match try!(self.curr()).tok {
                // Wildcard symbol -> Semi expected and return name.
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

    // fn eat_maybe(&mut self, t: Token) -> PResult<bool> {
    //     let curr = try!(self.curr());
    //     if curr.tok == t {
    //         self.bump();
    //         Ok(true)
    //     } else {
    //         Ok(false)
    //     }
    // }

    // fn expect_one_of(&mut self, eat: &[Token]/*, spare: &[Token]*/)
    //     -> PResult<TokenSpan> {
    //     let curr = try!(self.curr());
    //     if eat.contains(&curr.tok) {
    //         self.bump();
    //         Ok(curr)
    //     } else {
    //         Err(self.err_unexpected(eat, curr.tok))
    //     }
    // }

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
