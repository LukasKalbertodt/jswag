//! This module contains the lexing functionality
//!
//! The main type is the Tokenizer that will take the Java source code and
//! produces a sequence of tokens out of it.
//!
//! Most details of this module are defined in section 3 (lexical structure) of
//! the Java language specification.
//!

use std::str::Chars;
use std::iter::Iterator;
use super::token::*;
use diagnostics::ErrorHandler;
use filemap::{FileMap, Span, SrcIndex};
use std::str::FromStr;


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
    escaped_peek: u8,
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
            escaped_peek: 0,
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

        // update last
        self.last_pos = self.curr_pos;
        self.curr_pos = self.peek_pos;
        if let Some(c) = self.curr {
            self.peek_pos += if self.escaped_peek > 0 {
                self.escaped_peek as usize
            } else {
                c.len_utf8()
            };
            self.escaped_peek = 0;
        }

        // Check if the current char is a line break and add to line break list
        // Division into lines specified in section 3.4
        if self.curr == Some('\r')
            || (self.curr == Some('\n') && self.last != Some('\r')) {
            // we add the offset of the first char in the new line
            self.fmap.add_line(self.peek_pos);
        }

        // Check for unicode escape. More information in section 3.3
        if self.peek == Some('\\') && self.curr != Some('\\') {
            self.upeek = self.chs.next();
            if self.upeek == Some('u') {
                // First of all: We need `upeek` just for the case that the
                // char after '\' is not 'u'. So we reset it here.
                self.upeek = None;

                let mut pos_offset = 2;

                // We use a temporary peekable iterator here to check for
                // additional u's
                let mut peekiter = self.chs.by_ref().peekable();

                // After the first 'u' may follow arbitrarily many more 'u's...
                while let Some(&'u') = peekiter.peek() {
                    pos_offset += 1;
                    peekiter.next();
                }

                // At this point we expect 4 hexadecimal digits. Try to read
                // all four and convert them to int. We count the digits we
                // read in `num_digits`. If a non-hexadecimal char is found,
                // it'll be saved in `interrupt`.
                let mut value = 0;
                let mut num_digits = 0;
                let mut interrupt = None;
                for c in peekiter.by_ref().take(4) {
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
                            hi: self.peek_pos + pos_offset - 1 + num_digits,
                        },
                        "Invalid unicode escape (less than 4 digits)".into()
                    );
                    // ... but ignore the wrong unicode escape.
                    // If we couldn't read 4 digits because we reached EOF,
                    // interrupt will be None. If the reason was a non-hex
                    // char, it's saved in interrupt.
                    self.peek = interrupt;

                    // update position accordingly
                    self.peek_pos += pos_offset + num_digits;
                } else {
                    // we read all 4 digits and converted them to int. Now use
                    // that value to create a new char and save it into peek.
                    self.peek = match ::std::char::from_u32(value) {
                        Some(c) => {
                            self.escaped_peek = 4 + pos_offset as u8;
                            Some(c)
                        },
                        None => {
                            self.diag.span_err(
                                Span {
                                    lo: self.peek_pos,
                                    hi: self.peek_pos + pos_offset - 1 + 4,
                                },
                                "Invalid unicode escape (not a valid unicode \
                                    scalar value)".into()
                            );
                            self.peek_pos += pos_offset + 4;
                            peekiter.next()
                        }
                    };
                }
            }
        }

        // Check if peek is the SUB character to ignore it, if it is the last
        // character in the input stream (section 3.5)
        if self.peek == Some('\u{001a}') {
            self.upeek = self.chs.next();

            // If peek is the last char in the input stream, we ignore the
            // SUB character
            if self.upeek == None {
                self.peek = None
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
        while is_java_whitespace(self.curr.unwrap_or('x')) {
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
                !(self.curr == Some('*') && self.peek == Some('/')) {
                self.bump();
            }
            self.dbump();   // skip last two chars
        } else {
            // precondition tells us that `peek` is '/' here. Skip everything
            // until line break is reached.
            loop {
                match self.curr {
                    None | Some('\n') | Some('\r') => break,
                    _ => self.bump(),
                }
            }
        }
    }

    /// Scans a word and returns it as a `String`.
    ///
    /// This function scans through the source text as if we expect a Java
    /// identifier (Section 3.8). However, it might turn out to be a keyword,
    /// a boolean literal or a null literal.
    ///
    /// ## Preconditions
    /// `curr` needs to be a Java identifier start.
    fn scan_word(&mut self) -> Token {
        // Collect all chars until the first non-ident char or EOF is reached.
        let s: String = self.iter()
            .take_while(|&c| is_java_ident_part(c))
            .collect();

        // check if the word is a keyword
        match &s[..] {
            "true" => Token::Literal(Lit::Bool(true)),
            "false" => Token::Literal(Lit::Bool(false)),
            "null" => Token::Literal(Lit::Null),
            _ => match Keyword::from_str(&s) {
                Ok(k) => Token::KeyW(k),
                Err(_) => Token::Ident(s),
            },
        }
    }

    /// Reads a char inside a string or character literal. If `curr` is a
    /// backslash, the escape character is parsed, if possible. The boolean
    /// value denotes if the returned char was created from a escape sequence.
    ///
    /// Returns `None` if
    /// - `curr` is `None` (EOF),
    /// - a '\' followed by EOF is found
    ///
    /// Invalid escape sequences result in an error, but the backslash will be
    /// ignored and the char after it will be returned.
    fn scan_escaped_char(&mut self) -> (Option<char>, bool) {
        match self.curr {
            Some('\\') => {
                self.bump();
                (match self.curr {
                    None => None, // error, but it's handled somewhere else
                    Some('b') => { self.bump(); Some('\u{0008}') },
                    Some('t') => { self.bump(); Some('\t') },
                    Some('n') => { self.bump(); Some('\n') },
                    Some('f') => { self.bump(); Some('\u{000c}') },
                    Some('r') => { self.bump(); Some('\r') },
                    Some('\'') => { self.bump(); Some('\'') },
                    Some('\"') => { self.bump(); Some('\"') },
                    Some('\\') => { self.bump(); Some('\\') },
                    Some(c) => {
                        self.fatal_span(&format!("invalid escape sequence \
                            \\{}", c));
                        self.bump();
                        Some(c)
                    }
                }, true)
            }
            Some(c) => {
                self.bump();
                (Some(c), false)
            },
            None => (None, false),
        }
    }

    /// Scans a Java string literal.
    ///
    /// ## Preconditions
    /// `curr` needs to be `"`
    fn scan_string_literal(&mut self) -> String {
        self.bump();    // discard "

        let mut s = String::new();
        loop {
            match self.scan_escaped_char() {
                (Some('\"'), false) => break,
                (Some(c), _) => s.push(c),
                (None, _) => {
                    self.fatal_span("unexpected EOF in string literal");
                    break;
                },
            }
        }

        s
    }

    /// Scans a Java string literal.
    ///
    /// ## Preconditions
    /// `curr` needs to be `'`
    fn scan_char_literal(&mut self) -> char {
        self.bump();    // discard '
        if let (Some(c), escaped) = self.scan_escaped_char() {
            if c == '\'' && !escaped {
                self.fatal_span("empty character literal");
            } else {
                // we need another ' to close the literal
                if self.curr == Some('\'') {
                    self.bump();
                    return c;
                } else {
                    self.fatal_span("unclosed character literal");
                }
            }
        } else {
            self.fatal_span("unexpected EOF in character literal");
        }
        '\x00'
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
    /// `curr` needs to be in '0' ... '9' or a '.' followed by '0' ... '9'
    fn scan_number_literal(&mut self) -> Lit {
        let (r, s) = match self.curr {
            // if literal is starting with '0' (-> not decimal or single digit)
            Some('0') => {
                match self.peek.unwrap_or('\0') {
                    // hexadecimal literal
                    'x' | 'X' => {
                        self.dbump();   // skip "0x"
                        (16, if self.curr != Some('.') {
                            self.scan_digits(16)
                        } else {
                            "".into()
                        })
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
            // literal starting with a dot: decimal float. Note: No bump
            Some('.') => (10, "".into()),
            // literal starting with non-null digit (-> decimal)
            _ => (10, self.scan_digits(10))
        };

        // look at the first char after the whole number part
        match self.curr.unwrap_or('\0') {
            'l' | 'L' => {
                self.bump();
                Lit::Integer { raw: s, is_long: true, radix: r as u8 }
            },
            // After a whole number part may follow a float type suffix
            c @ 'f' | c @ 'F' | c @ 'd' | c @ 'D' if !s.is_empty() => {
                self.bump();
                Lit::Float {
                    raw: s,
                    is_double: (c != 'f' && c != 'F'),
                    radix: r as u8,
                    exp: "".into(),
                }
            },
            // If we have a whole number part, there may follow a exponent part
            'p' | 'P' | 'e' | 'E' if !s.is_empty() => {
                match self.scan_float_exp(r == 16) {
                    // Failing to scan the exponent means that the exponent
                    // char is wrong (p for hex, e for decimal)
                    None => {
                        if r == 16 {
                            self.fatal_span("invalid exponent indicator for \
                                hex float literal (use 'p' or 'P' instead");
                        } else {
                            self.fatal_span("invalid exponent indicator for \
                                decimal float literal (use 'f' or 'F' instead");
                        }
                        Lit::Integer {
                            raw: s, is_long: false, radix: r as u8
                        }
                    },
                    Some(ex) => {
                        // A float type suffix may follow
                        let double = self.scan_double_suffix().unwrap_or(true);

                        Lit::Float {
                            raw: s,
                            is_double: double,
                            radix: r as u8,
                            exp: ex,
                        }
                    }
                }
            }

            // A dot means float literal and may occur in two situations:
            // - we already read a whole number part
            // - the dot was the start of the literal
            '.' => {
                // make sure the literal is in the right base
                if r != 10 && r != 16 {
                    self.fatal_span(&format!("float literals may only be \
                        expressed in decimal or hexadecimal, not base {}", r));
                    return Lit::Integer {
                        raw: s, is_long: false, radix: r as u8
                    };
                }
                self.bump();    // dot

                // We do not need to check if both, the whole number and
                // fraction part, are empty in decimal case for the
                // following reason:
                // The precondition tells us that this function is only
                // called if `curr` is a number OR a dot followed by a
                // number. This guarantees that at least one part is
                // non-empty in decimal case.
                let fraction = self.scan_digits(r);
                if r == 16 && s.is_empty() && fraction.is_empty() {
                    self.fatal_span("hex float literals need either a \
                        whole number or a fraction part");
                }

                // In a hexadecimal float literal the exponent is required
                let exp = self.scan_float_exp(r == 16).unwrap_or("".into());
                if r == 16 && exp.is_empty() {
                    self.fatal_span("hex float literals are required to \
                        have an exponent");
                }

                // type suffix is always optional
                let is_double = self.scan_double_suffix().unwrap_or(true);
                Lit::Float {
                    raw: format!("{}.{}", s, fraction),
                    is_double: is_double,
                    radix: r as u8,
                    exp: exp,
                }
            },
            _ => Lit::Integer { raw: s, is_long: false, radix: r as u8 },
        }
    }

    /// Scans a float suffix ('d' or 'f') if present and returns if the
    /// suffix was a 'd' (double) suffix.
    fn scan_double_suffix(&mut self) -> Option<bool> {
        match self.curr.unwrap_or('\0') {
            'd' | 'D' => { self.bump(); Some(true) },
            'f' | 'F' => { self.bump(); Some(false) },
            _ => None
        }
    }

    /// Scans a float exponent
    fn scan_float_exp(&mut self, hex: bool) -> Option<String> {
        match (hex, self.curr.unwrap_or('\0')) {
            (false, 'e') | (false, 'E') | (true, 'p') | (true, 'P') => {
                self.bump();
                let minus = if self.curr == Some('-') {
                    self.bump();
                    "-"
                } else {
                    ""
                };
                Some(format!("{}{}", minus, self.scan_digits(10)))
            },
            _ => None,
        }
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
            match self.curr.unwrap_or('\0') {
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
            // non-real tokens: whitespace and comments
            c if is_java_whitespace(c) => {
                self.skip_whitespace();
                Token::Whitespace
            },
            '/' if p == '/' || p == '*' => {
                self.skip_comment();
                Token::Comment
            },

            // Java separators, ':' and float literals
            '(' => { self.bump(); Token::ParenOp },
            ')' => { self.bump(); Token::ParenCl },
            '{' => { self.bump(); Token::BraceOp },
            '}' => { self.bump(); Token::BraceCl },
            '[' => { self.bump(); Token::BracketOp },
            ']' => { self.bump(); Token::BracketCl },
            ';' => { self.bump(); Token::Semi },
            ',' => { self.bump(); Token::Comma },
            '.' => {
                match p {
                    '0' ... '9' => Token::Literal(self.scan_number_literal()),
                    _ => {
                        self.bump();
                        if p == '.' && self.peek == Some('.') {
                            self.dbump();
                            Token::DotDotDot
                        } else {
                            Token::Dot
                        }
                    }
                }
            },
            '@' => { self.bump(); Token::At },
            ':' if p == ':' => { self.dbump(); Token::ColonSep },
            ':' => { self.bump(); Token::Colon },

            // Operators  ==  =  >>>=  >>>  >>=  >>  >=  >  <<=  <<  <=  <
            '=' if p == '=' => { self.dbump(); Token::EqEq },
            '=' => { self.bump(); Token::Eq },
            '>' if p == '>' => {
                self.dbump();
                match self.curr.unwrap_or('\0') {
                    '>' => {
                        self.bump();
                        if self.curr == Some('=') {
                            self.bump();
                            Token::ShrUnEq
                        } else {
                            Token::ShrUn
                        }
                    },
                    '=' => {
                        self.bump();
                        Token::ShrEq
                    }
                    _ =>  {
                        Token::Shr
                    }
                }
            },
            '>' if p == '=' => { self.dbump(); Token::Ge },
            '>' => { self.bump(); Token::Gt },
            '<' if p == '<' => {
                self.dbump();
                if self.curr == Some('=') {
                    self.bump();
                    Token::ShlEq
                } else {
                    Token::Shl
                }
            },
            '<' if p == '=' => { self.dbump(); Token::Le },
            '<' => { self.bump(); Token::Lt },

            // Operators  !=  !  ~  ?
            '!' if p == '=' => { self.dbump(); Token::Ne },
            '!' => { self.bump(); Token::Bang },
            '~' => { self.bump(); Token::Tilde },
            '?' => { self.bump(); Token::Question },

            // Operators  +=  ++  +  -=  ->  --  -  &=  &&  &  |=  ||  |
            '+' if p == '=' => { self.dbump(); Token::PlusEq },
            '+' if p == '+' => { self.dbump(); Token::PlusPlus },
            '+' => { self.bump(); Token::Plus },
            '-' if p == '=' => { self.dbump(); Token::MinusEq },
            '-' if p == '>' => { self.dbump(); Token::Arrow },
            '-' if p == '-' => { self.dbump(); Token::MinusMinus },
            '-' => { self.bump(); Token::Minus },
            '&' if p == '=' => { self.dbump(); Token::AndEq },
            '&' if p == '&' => { self.dbump(); Token::AndAnd },
            '&' => { self.bump(); Token::And },
            '|' if p == '=' => { self.dbump(); Token::OrEq },
            '|' if p == '|' => { self.dbump(); Token::OrOr },
            '|' => { self.bump(); Token::Or },

            // Operators  *=  *  /=  /  %=  %  ^=  ^
            '*' if p == '=' => { self.dbump(); Token::StarEq },
            '*' => { self.bump(); Token::Star },
            '/' if p == '=' => { self.dbump(); Token::SlashEq },
            '/' => { self.bump(); Token::Slash },
            '^' if p == '=' => { self.dbump(); Token::CaretEq },
            '^' => { self.bump(); Token::Caret },
            '%' if p == '=' => { self.dbump(); Token::PercentEq },
            '%' => { self.bump(); Token::Percent },

            // Literals
            '"' => Token::Literal(Lit::Str(self.scan_string_literal())),
            '\'' => Token::Literal(Lit::Char(self.scan_char_literal())),
            '0' ... '9' => Token::Literal(self.scan_number_literal()),

            // identifier, keyword, bool- or null-literal
            c if is_java_ident_start(c) => {
                self.scan_word()
            },
            _ => {
                self.fatal_span("illegal character in this context");
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

/// Determines if the given character is a whitespace character as defined
/// in section 3.6
fn is_java_whitespace(c: char) -> bool {
    match c {
        ' ' | '\t' | '\u{000c}' | '\n' | '\r' => true,
        _ => false,
    }
}
