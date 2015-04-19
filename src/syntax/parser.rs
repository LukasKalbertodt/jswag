use std::iter::{Iterator};
use super::token::*;
use super::lexer::{Tokenizer, TokenSpan};
use std::boxed::Box;
use super::ast;
use filemap::Span;
use std::result::Result;
use diagnostics::ErrorHandler;
use std::mem::swap;
use std::collections::HashMap;
use std::default::Default;

enum PErr {
    Fatal,
}

pub type PResult<T> = Result<T, PErr>;

pub type ModifiersAtSpans = HashMap<ast::Modifier, Span>;


pub struct Parser<'a>{
    // Token stream
    tstr: Box<Tokenizer<'a>>,
    e: &'a ErrorHandler,

    // current token
    last: Option<TokenSpan>,
    curr: Option<TokenSpan>,
    peek: Option<TokenSpan>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Box<Tokenizer<'a>>, e: &'a ErrorHandler) -> Parser<'a> {
        let mut p = Parser {
            tstr: lex,
            e: e,
            last: None,
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
            items: Vec::new(),
        };
        loop {
            if self.curr.is_none() {
                break;
            }

            let curr = try!(self.curr());
            match curr.tok {
                Token::Keyword(Keyword::Import) => {
                    self.bump();
                    cu.items.push(ast::Item::Import(try!(self.parse_import())));
                },
                // TODO: Detecting beginning of class is more complex. It could
                // start with variouWass keywords and could be an interface.
                Token::Keyword(Keyword::Public) | Token::Keyword(Keyword::Class) => {
                    let boxed = Box::new(try!(self.parse_top_lvl_class()));
                    cu.items.push(ast::Item::Class(boxed));
                }
                _ => break,
            }
        }

        Ok(cu)
    }

    fn skip_block(&mut self, d: DelimToken) -> PResult<()> {
        // Just call the function if the next token is a '{'
        try!(self.eat(Token::OpenDelim(d)));
        let mut depth = 1;

        while depth > 0 {
            match self.curr {
                None => {
                    return Err(PErr::Fatal);
                },
                Some(ref curr) => {
                    match curr.tok {
                        Token::OpenDelim(delim) if delim == d => {
                            depth += 1;
                        },
                        Token::CloseDelim(delim) if delim == d => {
                            depth -= 1;
                        },
                        _ => {}
                    }
                },
            }
            self.bump()
        };
        Ok(())
    }

    fn parse_modifiers(&mut self) -> PResult<ModifiersAtSpans> {
        let mut mods = ModifiersAtSpans::new();

        loop {
            // Read each token and check if its a modifier token. If it is,
            // add the modifier associated with its span to the map. If the
            // map already contains that modifier, raise an error, since every
            // modifier is allowed only once.
            // Stop searching when the first non-modifier token appears.
            macro_rules! check_keywords {
                ($($k:ident,)*) => {{
                    // Modifiers are in front of other stuff, so there should
                    // be a next token (program is illformed otherwise).
                    let curr = try!(self.curr());
                    match curr.tok {
                        $( k @ Token::Keyword(Keyword::$k) => {
                            if mods.insert(ast::Modifier::$k, curr.span).is_some() {
                                return Err(self.err_dupe(k, curr.span));
                            }
                        }, )*
                        _ => break,
                    };
                }}
            }

            check_keywords!(Public, Private, Protected, Abstract, Static,
                Final, Synchronized, Native, Strictfp, Transient, Volatile,);

            // consume token
            self.bump();
        }

        Ok(mods)
    }

    fn parse_top_lvl_class(&mut self) -> PResult<ast::Class> {
        let mut c = ast::Class {
            name: ast::Ident::default(),
            vis: ast::Visibility::Package,
            methods: Vec::new(),
        };

        // Parse and verify class modifiers
        let mods = try!(self.parse_modifiers());
        for (m, s) in mods {
            match m {
                ast::Modifier::Public => {
                    c.vis = ast::Visibility::Public;
                },
                // TODO: Check and accept other modifiers
                o @ _ => {
                    self.e.span_err(s, format!("Unexpected class modifier `{}`", o));
                    return Err(PErr::Fatal);
                },
            }
        }

        // `class` is expected now.
        try!(self.eat(Token::Keyword(Keyword::Class)));

        // `class` was parsed, next token should be class name
        c.name = try!(self.eat_ident());

        // TODO: Type Params
        // TODO: Super class
        // TODO: implements

        // try!(self.skip_brace_block());

        // Start of class body
        try!(self.eat(Token::OpenDelim(DelimToken::Brace)));

        loop {
            // If a closing brace closes the class -> stop parsing
            if try!(self.eat_maybe(Token::CloseDelim(DelimToken::Brace))) {
                break;
            }

            // Try to parse a member. It starts with modifiers.
            let mmods = try!(self.parse_modifiers());

            // Next up will be a type
            let ty = try!(self.eat_ident());

            // Next up: Method name or first field name
            let name = try!(self.eat_ident());

            // At this point we can finally decide what we are parsing: If it's
            // a method, the next token needs to be a `(`. If it's a field it
            // could either be `;`, `=` or `,`.

            match try!(self.curr()).tok {
                Token::OpenDelim(DelimToken::Paren) => {
                    c.methods.push(try!(self.parse_method(name, ty, mmods)));
                },
                Token::Semi | Token::Eq | Token::Comma => {
                    while try!(self.curr()).tok != Token::Semi {
                        self.bump();
                    }
                    self.bump();
                }
                o @ _ => {
                    return Err(self.err_unexpected(
                        &[Token::OpenDelim(DelimToken::Paren), Token::Semi,
                        Token::Eq, Token::Comma], o));
                }
            }

        }

        Ok(c)
    }

    fn parse_method(&mut self, name: ast::Ident, ret_ty: ast::Ident, mods: ModifiersAtSpans)
        -> PResult<ast::Method> {
        let mut meth = ast::Method {
            vis: ast::Visibility::Package,
            name: name,
            ret_ty: ret_ty,
            static_: false,
            final_: false,
            params: Vec::new(),
        };

        // Parse and verify method modifiers ordered by span
        let mut mods_in_order : Vec<_> = mods.into_iter().collect();
        mods_in_order.sort_by(|a, b| a.1.lo.cmp(&b.1.lo));

        let mut parsed_vis : Option<(Span, ast::Modifier)> = None;
        for (m, s) in mods_in_order {
            match m {
                ast::Modifier::Public | ast::Modifier::Protected
                    | ast::Modifier::Private => {
                    match parsed_vis {
                        Some((span, vi)) => {
                            self.e.span_err(s, format!("Unexpected visibility modifier `{}`", m));
                            self.e.span_note(span,
                                format!("Already parsed the visibility modifier `{}` here", vi));
                            return Err(PErr::Fatal);
                        },
                        None => {
                            meth.vis = match m {
                                ast::Modifier::Public => ast::Visibility::Public,
                                ast::Modifier::Protected => ast::Visibility::Protected,
                                ast::Modifier::Private => ast::Visibility::Public,
                                _ => unreachable!(),
                            };
                            parsed_vis = Some((s, m));
                        }
                    }
                },
                ast::Modifier::Static => {
                    meth.static_ = true;
                },
                ast::Modifier::Final => {
                    meth.final_ = true;
                },
                // TODO: Check other modifiers (abstract, synchronized, native, strictfp)
                o @ _ => {
                    self.e.span_err(s, format!("Unexpected method modifier `{}`", o));
                    return Err(PErr::Fatal);
                },
            }
        }

        // parse parameter list
        // TODO: ReceiverParamter + LastFormalParameter
        try!(self.eat(Token::OpenDelim(DelimToken::Paren)));

        while !try!(self.eat_maybe(Token::CloseDelim(DelimToken::Paren))) {
            let mut param = ast::FormalParameter {
                ty: ast::Ident::default(),
                name: ast::Ident::default(),
                dims: 0,
                final_: false,
            };
            param.final_ = try!(self.eat_maybe(Token::Keyword(Keyword::Final)));
            param.ty = try!(self.eat_ident());  // type
            param.dims = try!(self.parse_dims());
            param.name = try!(self.eat_ident());  // name
            if param.dims == 0 {
                param.dims = try!(self.parse_dims());
            }
            try!(self.eat_maybe(Token::Comma));

            meth.params.push(param);
        }


        // try!(self.skip_block(DelimToken::Paren));

        // skip body
        try!(self.skip_block(DelimToken::Brace));

        Ok(meth)
    }

    fn parse_dims(&mut self) -> PResult<usize> {
        let mut count = 0;
        loop {
            if try!(self.eat_maybe(Token::OpenDelim(DelimToken::Bracket))) {
                try!(self.eat(Token::CloseDelim(DelimToken::Bracket)));
                count += 1;
            } else {
                break;
            }
        }
        Ok(count)
    }

    fn parse_import(&mut self) -> PResult<ast::Import> {
        // `import` has already been eaten.
        let mut name = ast::Name { path: Vec::new(), last: None };

        // The first token after `import` needs to be a word.
        let mut w = try!(self.eat_ident());

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
                f @ _ => return Err(self.err_unexpected(&[Token::Semi, Token::Dot], f)),
            }

            match try!(self.curr()).tok {
                // Wildcard symbol -> Semi expected and return name.
                Token::Star => {
                    self.bump();
                    try!(self.eat(Token::Semi));
                    return Ok(ast::Import::Wildcard(name));
                },
                Token::Word(..) => {
                    w = try!(self.eat_ident());
                },
                f @ _ => return Err(self.err_unexpected(
                    &[Token::Star, Token::Word("".to_string())], f)),
            }
        }
    }

    fn eat_ident(&mut self) -> PResult<ast::Ident> {
        let curr = try!(self.curr());

        match curr {
            TokenSpan { tok: Token::Word(name), span } => {
                self.bump();
                Ok(ast::Ident { name: name, span: span } )
            },
            _ => Err(self.err_unexpected(&[Token::Word("".to_string())], curr.clone().tok)),
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
        swap(&mut self.last, &mut self.curr);  // last = curr
        swap(&mut self.curr, &mut self.peek);  // curr = peek
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

    fn err_dupe(&self, t: Token, dupe_span: Span) -> PErr {
        self.e.span_err(dupe_span,
            format!("Duplicate token `{}`", t));
        PErr::Fatal
    }

    fn err_unexpected(&self, expected: &[Token], found: Token) -> PErr {
        let list = expected.iter().enumerate().fold(String::new(), |mut list, (idx, ref t)| {
            list.push_str(&*format!("`{}`", t));
            if idx + 2 < expected.len() {
                list.push_str(", ");
            } else if idx + 2 == expected.len() {
                list.push_str(" or ");
            }
            list
        });

        self.e.span_err(self.curr.clone().unwrap().span,
            format!("Unexpected token: Expected {}, found `{}`", list, found));
        PErr::Fatal
    }
}
