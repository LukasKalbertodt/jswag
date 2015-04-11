
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
pub enum Token {
    // Keyword(Keyword),
    Whitespace,
    Comment,
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

    fn scan_word(&mut self) -> String {
        let mut s = String::new();
        // Break if its whitespace or None (whitespace in that case)
        while !is_whitespace(self.curr.unwrap_or(' ')) {
            s.push(self.curr.unwrap());
            self.bump();
        }
        s
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = TokenSpan;

    fn next(&mut self) -> Option<TokenSpan> {
        let before_pos = self.curr_pos;

        match self.curr {
            None => None,
            Some(c) if is_whitespace(c) => {
                self.skip_whitespace();
                Some(TokenSpan {
                    tok: Token::Whitespace,
                    span: (before_pos, self.curr_pos)
                })
            },
            Some(c) if c == '/' => {
                match self.peek {
                    Some(c) if c == '/' || c == '*' => {
                        self.skip_comment();
                        Some(TokenSpan {
                            tok: Token::Comment,
                            span: (before_pos, self.curr_pos),
                        })
                    },
                    _ => {
                        Some(TokenSpan {
                            tok: Token::Other(self.scan_word()),
                            span: (before_pos, self.curr_pos),
                        })
                    }
                }
            },
            Some(_) => {
                Some(TokenSpan {
                    tok: Token::Other(self.scan_word()),
                    span: (before_pos, self.curr_pos),
                })
            }
        }




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
