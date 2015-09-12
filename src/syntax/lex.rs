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

    /// Buffered chars for easy access
    last: Option<char>,
    curr: Option<char>,
    peek: Option<char>,

    /// Byte offset of the corresponding char
    last_pos: SrcIndex,
    curr_pos: SrcIndex,
    peek_pos: SrcIndex,

    /// Byte offset when parsing the current token started
    token_start: SrcIndex,

    /// Used for translation of unicode escapes. Do not use directly
    upeek: Option<char>,
    peek_was_escaped: bool,
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
            peek_pos: 0,
            token_start: 0,
            upeek: None,
            peek_was_escaped: false,
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
        self.peek = if let Some(un) = self.upeek {
            self.upeek = None;
            Some(un)
        } else {
            self.chs.next()
        };

        // Check if the last char is a line break and add to break list
        if self.last == Some('\n') {
            self.fmap.add_line(self.peek_pos);
        }

        // update last
        self.last_pos = self.curr_pos;
        self.curr_pos = self.peek_pos;
        if let Some(c) = self.curr {
            self.peek_pos += if self.peek_was_escaped {
                self.peek_was_escaped = false;
                6
            } else {
                c.len_utf8()
            };
        }

        // check for unicode escape
        if self.peek == Some('\\') {
            self.upeek = self.chs.next();
            if self.upeek == Some('u') {
                // First of all: We need `upeek` just for the case that the
                // char after '\' is not 'u'. So we reset it here.
                self.upeek = None;

                // At this point we expect 4 hexadecimal digits. Try to read
                // all four and convert them to int. We count the digits we
                // read in `num_digits`. If a non-hexadecimal char is found,
                // it'll be saved in `interrupt`.
                let mut value = 0;
                let mut num_digits = 0;
                let mut interrupt = None;
                for c in self.chs.by_ref().take(4) {
                    match c.to_digit(16) {
                        Some(v) => {
                            // converting: Shifting left by 12, 8, 4 and 0.
                            value += v << ((3-num_digits)*4);
                            num_digits += 1;
                        },
                        None => {
                            interrupt = Some(c);
                            break;
                        },
                    }
                }

                // check if all four digits were supplied
                if num_digits < 4 {
                    // report error ...
                    self.diag.span_err(
                        Span {
                            lo: self.peek_pos,
                            hi: self.peek_pos + 1 + num_digits,
                        },
                        "Invalid unicode escape (less than 4 digits)".into()
                    );
                    // ... but ignore the wrong unicode escape.
                    // If we couldn't read 4 digits because we reached EOF,
                    // interrupt will be None. If the reason was a non-hex
                    // char, it's saved in interrupt.
                    self.peek = interrupt;

                    // update position accordingly
                    self.peek_pos += 2 + num_digits;
                } else {
                    // we read all 4 digits and converted them to int. Now use
                    // that value to create a new char and save it into peek.
                    self.peek = match ::std::char::from_u32(value) {
                        Some(c) => {
                            self.peek_was_escaped = true;
                            Some(c)
                        },
                        None => {
                            self.diag.span_err(
                                Span {
                                    lo: self.peek_pos,
                                    hi: self.peek_pos + 5,
                                },
                                "Invalid unicode escape (not a valid unicode \
                                    scalar value)".into()
                            );
                            self.peek_pos += 6;
                            self.chs.next()
                        }
                    };
                }
            }
        }

        // println!("AFTER: {:?} {:?} {:?} || {} {}",
        //     self.last, self.curr, self.peek,
        //     self.curr_pos, self.peek_pos);
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
                hi: self.peek_pos
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
        if self.peek == Some('*') {
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
    fn scan_ident(&mut self) -> String {
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

    /// Scans a Java number literal and returns it. The literal may be a float
    /// or a integer literal. See section 3.10.1 and 3.10.2.
    ///
    /// Parsing the string as a number could be done at this point in theory.
    /// I need to think about it to find out if it makes sense.
    ///
    /// # Note
    /// Parsing of floating point literals is not ready yet!
    ///
    /// ## Preconditions
    /// `curr` needs to be in '0' ... '9'.
    fn scan_number_literal(&mut self) -> Lit {
        let (r, s) = match self.curr {
            // if literal is starting with '0' (-> not decimal or single digit)
            Some('0') => {
                match self.peek.unwrap_or('\0') {
                    // hexadecimal literal
                    'x' | 'X' => {
                        self.dbump();   // skip "0x"
                        (16, self.scan_digits(16))
                    },
                    // binary literal
                    'b' | 'B' => {
                        self.dbump();   // skip "0b"
                        (2, self.scan_digits(2))
                    },
                    // octal literal
                    '0' ... '9' => {
                        self.bump();   // skip "0"
                        (8, self.scan_digits(8))
                    },
                    // just a '0'
                    _ => {
                        self.bump();
                        (10, "0".into())
                    }
                }
            },
            // literal starting with non-null digit (-> decimal)
            _ => (10, self.scan_digits(10))
        };

        // peek at the first char after the number for suffix detection
        let mut long_suffix = false;
        match self.curr.unwrap_or('0') {
            'l' | 'L' => {
                self.bump();
                long_suffix = true;
            },
            _ => {},
        }

        Lit::Integer(s, long_suffix, r as u8)
    }

    /// Scans digits with the given radix and returns the scanned string.
    ///
    /// The parsing will skip underscores and will stop when a character is
    /// found, that is no digit in the given radix. However, if the radix is
    /// less than 10, all digits from 0 to 9 are scanned and an error is
    /// printed for each digit that is too high for the given radix.
    fn scan_digits(&mut self, radix: u32) -> String {
        // We possibly scan more digits to report smart errors
        let scan_radix = if radix <= 10 { 10 } else { radix };

        let mut s = String::new();
        loop {
            match self.curr.unwrap_or('*') {
                // skip underscores
                '_' => {
                    self.bump();
                    continue;
                },
                c if c.to_digit(scan_radix).is_some() => {
                    // check if the digit is valid in the given radix
                    // TODO: Maybe stop lexing here
                    if c.to_digit(radix).is_none() {
                        self.diag.span_err(
                            Span { lo: self.curr_pos, hi: self.curr_pos },
                            format!("Invalid digit for base {} literal", radix)
                        );
                    }
                    s.push(c);
                    self.bump();
                }
                _ => return s,
            }
        }
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
            '!' => { self.bump(); Token::Bang },
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
            '0' ... '9' => Token::Literal(self.scan_number_literal()),

            'a' ... 'z' | 'A'... 'Z' => {
                let w = self.scan_ident();
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
            span: Span { lo: self.token_start, hi: self.last_pos },
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
