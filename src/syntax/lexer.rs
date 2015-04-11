use std::str::Chars;
use std::iter::{Iterator};
use super::token::*;
use diagnostics::ErrorHandler;
use filemap::FileMap;
use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub tok: Token,
    /// Byte position, half open: inclusive-exclusive
    pub span: (u64, u64),
    pub line: u64,
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
    last_pos: u64,
    /// Byte offset of the next character to read (peek)
    curr_pos: u64,

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
            fatal: false,
        };
        // tok.chs = tok.fmap.src.chars();
        tok.dbump();
        tok
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
            Some(c) => self.curr_pos += c.len_utf8() as u64,
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

    fn fatal(&mut self, m: &str) {
        self.diag.error(m);
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

    fn scan_string(&mut self) -> String {
        let mut s = String::new();
        // Break if its whitespace or None (whitespace in that case)
        loop {
            match self.curr.unwrap_or(' ') {
                c if is_whitespace(c) => break,
                c => {
                    s.push(c);
                },
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
                    self.fatal("Unexpected EOF while parsing string literal");
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
        let before_pos = self.curr_pos;
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
                    Token::BinOpEq(BinOpToken::Shl)
                } else {
                    Token::BinOp(BinOpToken::Shl)
                }
            },
            '<' => { self.bump(); Token::Lt },
            '>' if p == '=' => { self.dbump(); Token::Ge },
            '>' if p == '>' => {
                self.dbump();
                self.bump();
                if self.last.unwrap_or('x') == '=' {
                    Token::BinOpEq(BinOpToken::Shl)
                } else {
                    Token::BinOp(BinOpToken::Shl)
                }
            },
            '>' => { self.bump(); Token::Gt },

            '+' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::Plus)},
            '+' => { self.bump(); Token::BinOp(BinOpToken::Plus)},
            '-' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::Minus)},
            '-' => { self.bump(); Token::BinOp(BinOpToken::Minus)},
            '*' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::Star)},
            '*' => { self.bump(); Token::BinOp(BinOpToken::Star)},
            '/' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::Slash)},
            '/' => { self.bump(); Token::BinOp(BinOpToken::Slash)},
            '%' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::Percent)},
            '%' => { self.bump(); Token::BinOp(BinOpToken::Percent)},
            '^' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::Caret)},
            '^' => { self.bump(); Token::BinOp(BinOpToken::Caret)},
            '&' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::And)},
            '&' if p == '&' => { self.dbump(); Token::AndAnd },
            '&' => { self.bump(); Token::BinOp(BinOpToken::And)},
            '|' if p == '=' => { self.dbump(); Token::BinOpEq(BinOpToken::Or)},
            '|' if p == '|' => { self.dbump(); Token::OrOr },
            '|' => { self.bump(); Token::BinOp(BinOpToken::Or)},
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
            _ => Token::Other(self.scan_string()),
        };

        // println!("{:?}", self.line_breaks);
        // panic!("yolo");

        if self.fatal {
            return None;
        }

        Some(TokenSpan {
            tok: t,
            span: (before_pos, self.curr_pos),
            line: (self.fmap.num_lines() as u64) + 1,
        })


        // match self.curr {
        //     Some(_) => Some(Token {
        //         ty: TokenType::Other(s),
        //         span: (0, 0),
        //     }),
        //     None => None,
        // }
        // self.chs.next()
    }
}


fn is_whitespace(c: char) -> bool {
    match c {
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false,
    }
}
