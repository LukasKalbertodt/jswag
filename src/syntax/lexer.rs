use std::str::Chars;
use std::iter::{Iterator};
use super::token::*;
use diagnostics::ErrorHandler;
use filemap::{FileMap, Span, SrcIndex};
use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub tok: Token,
    /// Byte position of token in Filemap
    pub span: Span,
}


pub struct Tokenizer<'a> {
    fmap: Rc<FileMap>,
    chs: Chars<'a>,

    // line_breaks: Vec<i64>,

    diag: &'a ErrorHandler,

    last: Option<char>,
    curr: Option<char>,
    peek: Option<char>,
    /// Byte offset of the last character read (curr)
    last_pos: SrcIndex,
    /// Byte offset of the next character to read (peek)
    curr_pos: SrcIndex,
    /// Byte offset when parsing the current token started
    token_start: SrcIndex,

    fatal: bool,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new Tokenizer from a string.
    pub fn new(fmap: &'a Rc<FileMap>, diag: &'a ErrorHandler) -> Tokenizer<'a> {
        let iter = fmap.src.chars();
        let mut tok = Tokenizer {
            chs: iter,
            fmap: fmap.clone(),
            // line_breaks: Vec::new(),
            diag: diag,
            last: None,
            curr: None,
            peek: None,
            last_pos: 0,
            curr_pos: 0,
            token_start: 0,
            fatal: false,
        };
        // tok.chs = tok.fmap.src.chars();
        tok.dbump();
        tok
    }

    pub fn next_real(&mut self) -> Option<TokenSpan> {
        loop {
            match self.next() {
                Some(ref t) if !t.tok.is_real() => {},
                t @ _ => return t,
            }
        }
    }

    /// Reads a new char from the iterator, updating last, curr and peek + pos
    fn bump(&mut self) {
        self.last = self.curr;
        self.curr = self.peek;
        self.peek = self.chs.next();

        // Check if the last char is a line break and add to break list
        if self.last.unwrap_or('x') == '\n' {
            self.fmap.add_line(self.curr_pos);
        }

        self.last_pos = self.curr_pos;
        match self.peek {
            Some(c) => self.curr_pos += c.len_utf8(),
            _ => {}
        };

    }

    /// Calls `bump` twice. For less typing.
    fn dbump(&mut self) {
        self.bump();
        self.bump();
    }

    /// Calls `bump` until the first non-whitespace char is reached.
    fn skip_whitespace(&mut self) {
        while is_whitespace(self.curr.unwrap_or('x')) {
            self.bump();
        }
    }

    // fn fatal(&mut self, m: &str) {
    //     self.diag.error(m);
    //     self.fatal = true;
    // }

    fn fatal_span(&mut self, m: &str) {
        self.diag.span_err(Span { lo: self.token_start, hi: self.curr_pos}, m);
        self.fatal = true;
    }

    /// Calls `bump` until the first char after the comment is reached. Skips
    /// `/* */` and `//` comments.
    fn skip_comment(&mut self) {
        // We know curr == '/' and peek is either '*' or '/'.
        // Note: `self.peek.is_some()` implies `self.curr.is_some()`
        if self.peek.unwrap() == '*' {
            while self.peek.is_some() &&
                !(self.curr.unwrap() == '*' && self.peek.unwrap() == '/') {
                self.bump();
            }
            self.dbump();   // skip last two chars
        } else {
            while self.curr.is_some() && self.curr.unwrap() != '\n' {
                self.bump();
            }
        }
    }

    fn scan_real(&mut self) -> String {
        let mut s = String::new();
        // Break if its whitespace or None (whitespace in that case)
        loop {
            match self.curr.unwrap_or(' ') {
                'a' ... 'z' | 'A' ... 'Z' => {
                    s.push(self.curr.unwrap());
                },
                _ => break,
            }
            self.bump();
        }
        s
    }

    fn scan_string_literal(&mut self) -> String {
        // TODO: Escape shit
        // `curr` is '"'. Note: After one bump, `last` != None
        self.bump();

        let mut s = String::new();
        loop {
            match self.curr {
                Some(c) if c == '"' && self.last.unwrap() != '\\' => break,
                None => {
                    self.fatal_span("Unexpected EOF while lexing string literal");
                    break;
                },
                Some(c) => {
                    s.push(c);
                    self.bump();
                },
            }
        }
        self.bump();    // Remove last '"'
        s
    }

    /// Scans a Java integer literal and returns it as a String. Parsing the
    /// string as a number cannot be done at this point.
    /// There are three types of integer literals in Java:
    /// * `26`: Decimal
    /// * `0x1a`: Hex
    /// * `0b11010`: Binary
    /// All types can have an 'l' or 'L' suffix (-> type long, int otherwise)
    fn scan_integer_literal(&mut self) -> String {
        // `curr` is '0' ... '9' here.

        let mut s = String::new();

        match self.peek {
            Some(c) if (c == 'x' || c == 'X') && self.curr.unwrap() == '0' => {
                s.push('0');
                s.push(c);
                self.dbump();

                loop {
                    match self.curr {
                        Some(c) => match c {
                            '0' ... '9' | 'a' ... 'f' | 'A' ... 'F' => {
                                s.push(c);
                                self.bump();
                            },
                            _ => break,
                        },
                        _ => break,
                    }
                }
            },
            Some(c) if (c == 'b' || c == 'B') && self.curr.unwrap() == '0' => {
                s.push('0');
                s.push(c);
                self.dbump();

                loop {
                    match self.curr {
                        Some(c) if c == '0' || c == '1' => {
                            s.push(c);
                            self.bump();
                        },
                        _ => break,
                    }
                }
            },
            // `peek` can actually be anything, e.g. ';' or ' ', even None
            _ => {
                loop {
                    match self.curr {
                        Some(c) => match c {
                            '0' ... '9' => {
                                s.push(c);
                                self.bump();
                            },
                            _ => break,
                        },
                        _ => break,
                    }
                }
            },
        }

        match self.curr {
            Some(c) if c == 'l' || c == 'L' => s.push(c),
            _ => {},
        };

        s
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = TokenSpan;

    fn next(&mut self) -> Option<TokenSpan> {
        self.token_start = self.curr_pos;
        let p = self.peek.unwrap_or('\x00');

        if self.curr.is_none() {
            return None;
        }

        let t = match self.curr.unwrap() {
            c if is_whitespace(c) => {
                self.skip_whitespace();
                Token::Whitespace
            },
            '/' if p == '/' || p == '*' => {
                self.skip_comment();
                Token::Comment
            },
            '.' => { self.bump(); Token::Dot },
            ',' => { self.bump(); Token::Comma },
            ';' => { self.bump(); Token::Semi },

            '(' => { self.bump(); Token::OpenDelim(DelimToken::Paren) },
            ')' => { self.bump(); Token::CloseDelim(DelimToken::Paren) },
            '[' => { self.bump(); Token::OpenDelim(DelimToken::Bracket) },
            ']' => { self.bump(); Token::CloseDelim(DelimToken::Bracket) },
            '{' => { self.bump(); Token::OpenDelim(DelimToken::Brace) },
            '}' => { self.bump(); Token::CloseDelim(DelimToken::Brace) },

            '=' if p == '=' => { self.dbump(); Token::EqEq },
            '=' => { self.bump(); Token::Eq },
            '!' if p == '=' => { self.dbump(); Token::Ne },
            '!' => { self.bump(); Token::Not },
            '<' if p == '=' => { self.dbump(); Token::Le },
            '<' if p == '<' => {
                self.dbump();
                self.bump();
                if self.last.unwrap_or('x') == '=' {
                    Token::ShlEq
                } else {
                    Token::Shl
                }
            },
            '<' => { self.bump(); Token::Lt },
            '>' if p == '=' => { self.dbump(); Token::Ge },
            '>' if p == '>' => {
                self.dbump();
                self.bump();
                if self.last.unwrap_or('x') == '=' {
                    Token::ShrEq
                } else {
                    Token::Shr
                }
            },
            '>' => { self.bump(); Token::Gt },

            '+' if p == '=' => { self.dbump(); Token::PlusEq },
            '+' => { self.bump(); Token::Plus },
            '-' if p == '=' => { self.dbump(); Token::MinusEq },
            '-' => { self.bump(); Token::Minus },
            '*' if p == '=' => { self.dbump(); Token::StarEq },
            '*' => { self.bump(); Token::Star },
            '/' if p == '=' => { self.dbump(); Token::SlashEq },
            '/' => { self.bump(); Token::Slash },
            '%' if p == '=' => { self.dbump(); Token::PercentEq },
            '%' => { self.bump(); Token::Percent },
            '^' if p == '=' => { self.dbump(); Token::CaretEq },
            '^' => { self.bump(); Token::Caret },
            '&' if p == '=' => { self.dbump(); Token::AndEq },
            '&' if p == '&' => { self.dbump(); Token::AndAnd },
            '&' => { self.bump(); Token::And },
            '|' if p == '=' => { self.dbump(); Token::OrEq },
            '|' if p == '|' => { self.dbump(); Token::OrOr },
            '|' => { self.bump(); Token::Or },
            '~' => { self.bump(); Token::Tilde }

            '"' => Token::Literal(Lit::Str(self.scan_string_literal())),
            '0' ... '9' => Token::Literal(Lit::Integer(self.scan_integer_literal())),

            'a' ... 'z' | 'A'... 'Z' => {
                let w = self.scan_real();
                match Keyword::from_str(&w) {
                    Some(kw) => Token::Keyword(kw),
                    None => Token::Word(w),
                }
            },
            _ => {
                self.fatal_span("Could not lex string");
                Token::Whitespace   // Dummy token
            },
        };

        if self.fatal {
            return None;
        }

        Some(TokenSpan {
            tok: t,
            span: Span { lo: self.token_start, hi: self.curr_pos },
        })
    }
}


fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false,
    }
}
