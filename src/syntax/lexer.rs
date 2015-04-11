use std::str::Chars;
use std::iter::{Iterator};
use super::token::*;


#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub tok: Token,
    /// Byte position, half open: inclusive-exclusive
    pub span: (i64, i64),
    // pub line: i64,
}


pub struct Tokenizer<'a> {
    chs: Chars<'a>,

    // line_breaks: Vec<i64>,

    last: Option<char>,
    curr: Option<char>,
    peek: Option<char>,
    curr_pos: i64,
    peek_pos: i64,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new Tokenizer from a string.
    pub fn new(content: &'a String) -> Tokenizer<'a> {
        let iter = content.chars();
        let mut tok = Tokenizer {
            chs: iter,
            // line_breaks: Vec::new(),
            last: None,
            curr: None,
            peek: None,
            curr_pos: -1,
            peek_pos: -1,
        };
        tok.dbump();
        tok
    }

    /// Reads a new char from the iterator, updating last, curr and peek + pos
    fn bump(&mut self) {
        self.last = self.curr;
        self.curr = self.peek;
        self.peek = self.chs.next();

        self.curr_pos = self.peek_pos;
        match self.peek {
            Some(c) => self.peek_pos += c.len_utf8() as i64,
            _ => {}
        };
    }

    /// Calls `bump` twice. For less typing.
    fn dbump(&mut self) {
        self.bump();
        self.bump();
    }

    /// Calls `bump` until the first non-whitespace char is reached. Returns
    /// true if any newline char was skipped
    fn skip_whitespace(&mut self) -> bool {
        let mut newline_found = false;
        while is_whitespace(self.curr.unwrap_or('x')) {
            if self.curr.unwrap_or('x') == '\n' && !newline_found {
                newline_found = true;
            }
            self.bump();
        }
        newline_found
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
        // `curr` is '"'. Note: After one bump, `last` != None
        self.bump();

        let mut s = String::new();
        loop {
            match self.curr {
                Some(c) if c == '"' && self.last.unwrap() != '\\' => break,
                None => break,  // TODO: This should not happen!
                Some(c) => {
                    s.push(c);
                    self.bump();
                },
            }
        }
        self.bump();    // Remove last '"'
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
                Token::Whitespace(self.skip_whitespace())
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

            '"' => Token::Literal(Lit::Str(self.scan_string_literal())),

            'a' ... 'z' | 'A'... 'Z' => {
                let w = self.scan_real();
                match Keyword::from_str(&w) {
                    Some(kw) => Token::Keyword(kw),
                    None => Token::Word(w),
                }
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
