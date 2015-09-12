use std::str::Chars;
use std::iter::{Iterator};
use super::token::*;
use diagnostics::ErrorHandler;
use filemap::{FileMap, Span, SrcIndex};
use std::rc::Rc;


pub enum Error {
    Fatal
}


/// The Java Tokenizer.
///
/// This type takes the Java source code as string and produces a sequence of
/// `Token`s. It reads the source string from front to back only once. During
/// tokenization it will also detect newline characters and notifies the
/// filemap about them.
///
/// It implements the `Iterator` trait, yielding `TokenSpan`s.
pub struct Tokenizer<'a> {
    /// Filemap containing the whole source code
    fmap: &'a FileMap,
    /// Iterator into the filemaps's source code to easily obtain chars
    chs: Chars<'a>,
    /// Error handler to report errors during lexing
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
}

impl<'a> Tokenizer<'a> {
    // =======================================================================
    // Public methods of the Tokenizer
    // =======================================================================
    /// Creates a new Tokenizer with a reference to a filemap and to an error
    /// handler to report errors.
    pub fn new(fmap: &'a FileMap, diag: &'a ErrorHandler) -> Tokenizer<'a> {
        let mut tok = Tokenizer {
            chs: fmap.src.chars(),
            fmap: fmap,
            diag: diag,
            last: None,
            curr: None,
            peek: None,
            last_pos: 0,
            curr_pos: 0,
            token_start: 0,
        };
        tok.dbump();
        tok
    }

    /// Works like `next()` but will skip non-real Token (Whitespace, ...)
    pub fn next_real(&mut self) -> Option<TokenSpan> {
        self.find(|t| t.tok.is_real())
    }

    // =======================================================================
    // Private helper methods
    // =======================================================================
    /// Reads a new char from the iterator, updating last, curr and peek + pos
    fn bump(&mut self) {
        self.last = self.curr;
        self.curr = self.peek;
        self.peek = self.chs.next();

        // Check if the last char is a line break and add to break list
        if let Some('\n') = self.last {
            self.fmap.add_line(self.curr_pos);
        }

        // update last
        self.last_pos = self.curr_pos;
        if let Some(c) = self.peek {
            self.curr_pos += c.len_utf8();
        }
    }

    /// Calls `bump` twice. For less typing.
    fn dbump(&mut self) {
        self.bump();
        self.bump();
    }

    /// Convenience function to return a char iterator over `curr`. See
    /// `CharIter` for more info.
    fn iter<'b>(&'b mut self) -> CharIter<'b, 'a> {
        CharIter::new(self)
    }

    /// Prints a fatal error message through error diagnostic
    fn fatal_span(&mut self, m: &str) {
        self.diag.span_err(
            Span {
                lo: self.token_start,
                hi: self.curr_pos
            },
            m.to_string()
        );
    }

    // =======================================================================
    // Private skip and scan methods
    // =======================================================================
    /// Calls `bump` until the first non-whitespace char or EOF is reached.
    fn skip_whitespace(&mut self) {
        while self.curr.unwrap_or('x').is_whitespace() {
            self.bump();
        }
    }

    /// Skips `/* */` and `//` comments. Calls `bump` until the first char
    /// after the comment is reached.
    ///
    /// ## Preconditions
    /// `curr` needs to be '/' and `peek` needs to be one of '*' and '/'
    fn skip_comment(&mut self) {
        // Note: `self.peek.is_some()` implies `self.curr.is_some()`
        if let Some('*') = self.peek {
            // Skip everything until the end of file or a "*/" is reached.
            while self.peek.is_some() &&
                !(self.curr.unwrap() == '*' && self.peek.unwrap() == '/') {
                self.bump();
            }
            self.dbump();   // skip last two chars
        } else {
            // precondition tells us that `peek` is '/' here. Skip everything
            // until line break is reached.
            while self.curr.unwrap_or('\n') != '\n' {
                self.bump();
            }
        }
    }

    /// Scans a Java Identifier and returns it as a `String`. Section 3.8.
    ///
    /// ## Preconditions
    /// `curr` needs to be a Java identifier start.
    fn scan_real(&mut self) -> String {
        // Collect all chars until the first non-ident char or EOF is reached.
        self.iter().take_while(|&c| is_java_ident_part(c)).collect()
    }

    /// Scans a string literal. Not finished yet!
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
            c if c.is_whitespace() => {
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
                return None;
            },
        };

        Some(TokenSpan {
            tok: t,
            span: Span { lo: self.token_start, hi: self.curr_pos },
        })
    }
}

// ===========================================================================
// Definition of a helper iterator
// ===========================================================================
/// Helper type to iterate over the current chars of the tokenizer. Only used
/// by the tokenizer itself.
///
/// Whenever the iterator yields a char 'c', it is equal to `origin.curr`. That
/// means that after using the iterator, `curr` of the original tokenizer
/// equals the last char yielded by the iterator. However, the first char
/// yielded is also `curr` of the tokenizer.
/// It means that calling `next` n times will call `origin.bump()` n-1 times.
struct CharIter<'tok, 's: 'tok> {
    origin: &'tok mut Tokenizer<'s>,
    first: bool,
}

impl<'tok, 's> CharIter<'tok, 's> {
    fn new(orig: &'tok mut Tokenizer<'s>) -> CharIter<'tok, 's> {
        CharIter {
            origin: orig,
            first: true,
        }
    }
}

impl<'tok, 's> Iterator for CharIter<'tok, 's> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if self.first {
            self.first = false;
        } else {
            self.origin.bump();
        }
        self.origin.curr
    }
}

// ===========================================================================
// A bunch of helper functions
// ===========================================================================

/// Determines if the given character is a valid Java identifier start char
/// (`JavaLetter`, section 3.8.).
///
/// This function does not match the exact definition! However, it returns the
/// same values for all chars from `\u{0}` up to at least `\x{400}` as the
/// function `isJavaIdentifierStart` from "openjdk8".
fn is_java_ident_start(c: char) -> bool {
    match c {
        '$' | '_' | '¢' | '£' | '¤' | '¥' => true,
        '\u{345}' | '\u{37f}' => false,
        _ => c.is_alphabetic(),
    }
}

/// Determines if the given character is a valid Java identifier char
/// (`JavaLetterOrDigit`, section 3.8.).
///
/// This function does not match the exact definition! However, it returns the
/// same values for all chars from `\u{0}` up to at least `\x{400}` as the
/// function `isJavaIdentifierPart` from "openjdk8".
fn is_java_ident_part(c: char) -> bool {
    match c {
        '\u{ad}'
            | '\u{000}' ... '\u{008}'
            | '\u{00e}' ... '\u{01b}'
            | '\u{07f}' ... '\u{09f}'
            | '\u{300}' ... '\u{374}' => true,
        '\u{37f}' => false,
        _ => c.is_numeric() || is_java_ident_start(c),
    }
}
