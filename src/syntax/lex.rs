
use std::str::Chars;
use std::iter::{Iterator};

// macro_rules! declare_keywords {(
//     $( ($name:ident, $word:expr); )*
// ) => {
//     #[derive(Copy, Clone, PartialEq, Eq)]
//     pub enum Keyword {
//         $( $name, )*
//     }

//     impl Keyword {
//         pub fn word(&self) -> &'static str {
//             match *self {
//                 $( Keyword::$name => $word, )*
//             }
//         }
//     }
// }}

// declare_keywords! {
//     (Public,        "public");
//     (Private,       "private");
//     (Protected,     "protected");
//     (Class,         "class");
//     (Static,        "static");
// }

#[derive(Debug, Clone)]
pub enum DelimToken {
    Paren,      // round ( )
    Bracket,    // square [ ]
    Brace,      // curly { }
}

#[derive(Debug, Clone)]
pub enum Token {
    // Keyword(Keyword),
    Whitespace,
    Comment,
    Dot,
    OpenDelim(DelimToken),
    CloseDelim(DelimToken),
    Other(String),
}

impl Token {
    pub fn is_real(&self) -> bool {
        match *self {
            Token::Whitespace => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct TokenSpan {
    pub tok: Token,
    /// Byte position, half open: inclusive-exclusive
    pub span: (i64, i64),
}



pub struct Tokenizer<'a> {
    chs: Chars<'a>,

    curr: Option<char>,
    peek: Option<char>,
    curr_pos: i64,
    peek_pos: i64,
}

impl<'a> Tokenizer<'a> {
    pub fn new(content: &'a String) -> Tokenizer<'a> {
        let iter = content.chars();
        let mut tok = Tokenizer {
            chs: iter,
            curr: None,
            peek: None,
            curr_pos: -1,
            peek_pos: -1,
        };
        tok.bump();
        tok.bump();
        tok
    }

    fn bump(&mut self) {
        self.curr = self.peek;
        self.curr_pos = self.peek_pos;
        self.peek = self.chs.next();
        match self.peek {
            Some(c) => self.peek_pos += c.len_utf8() as i64,
            _ => {}
        };
    }

    fn skip_whitespace(&mut self) {
        while self.curr.is_some() && is_whitespace(self.curr.unwrap()) {
            self.bump();
        }
    }

    fn skip_comment(&mut self) {
        // We know curr == '/' and peek is either '*' or '/'.
        // Note: `self.peek.is_some()` implies `self.curr.is_some()`
        if self.peek.unwrap() == '*' {
            while self.peek.is_some() &&
                !(self.curr.unwrap() == '*' && self.peek.unwrap() == '/') {
                self.bump();
            }
            self.bump();
            self.bump();
        } else {
            while self.curr.is_some() && self.curr.unwrap() != '\n' {
                self.bump();
            }
            self.bump();
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
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = TokenSpan;

    fn next(&mut self) -> Option<TokenSpan> {
        let before_pos = self.curr_pos;

        if self.curr.is_none() {
            return None;
        }

        let t = match (self.curr.unwrap(), self.peek.unwrap_or('\x00')) {
            (c, _) if is_whitespace(c) => {
                self.skip_whitespace();
                Token::Whitespace
            },
            ('/', '/') | ('/', '*') => {
                self.skip_comment();
                Token::Comment
            },
            ('.', _) => {
                self.bump();
                Token::Dot
            },
            ('(', _) => { self.bump(); Token::OpenDelim(DelimToken::Paren) },
            (')', _) => { self.bump(); Token::CloseDelim(DelimToken::Paren) },
            ('[', _) => { self.bump(); Token::OpenDelim(DelimToken::Bracket) },
            (']', _) => { self.bump(); Token::CloseDelim(DelimToken::Bracket) },
            ('{', _) => { self.bump(); Token::OpenDelim(DelimToken::Brace) },
            ('}', _) => { self.bump(); Token::CloseDelim(DelimToken::Brace) },
            (c, _) if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') => {
                Token::Other(self.scan_real())
            },
            _ => Token::Other(self.scan_string()),
        };

        Some(TokenSpan {
            tok: t,
            span: (before_pos, self.curr_pos)
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
