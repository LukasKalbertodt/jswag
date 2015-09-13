//! This module defines basic token types
//!
//! The definition of Java tokens is mostly in section 3 (lexical structure) of
//! the Java language specification.
//!

// TODO: Remove
#![allow(dead_code)]

use std::fmt::{Display, Formatter, Error};
use filemap::Span;


/// A token with it's span in the source text
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSpan {
    pub tok: Token,
    /// Byte position of token in Filemap
    pub span: Span,
}

/// A Java token
///
/// This enum differs a bit from the original definition in the Java spec, in
/// which this `Token` is called *InputElement* and is defined as:
/// ```
/// WhiteSpace  |  Comment  |  Token
/// ```
/// The Java-*Token* is defined as:
/// ```
/// Identifier  |  Keyword  |  Literal  |  Seperator  |  Operator
/// ```
///
/// This `Token` type differs from the formal and correct definition to make
/// the parser and lexer module less verbose. The differences are:
/// - all 5 variants of the Java-*Token* are direct variants of this `Token`
/// - therefore the name Java-*Token* is not necessary and Java's
///   *InputElement* is called `Token` instead
/// - *Seperator*s and *Operator*s are also direct variants of this `Token`
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Variants of the Java-*InputElement* (not "real" tokens)
    Whitespace,
    Comment,

    // Variants of the Java-*Token*
    Ident(String),
    Keyword(Keyword),
    Literal(Lit),

    // Variants of Java-*Seperator*
    // (   )   {   }   [   ]   ;   ,   .   ...   @   ::
    ParenOp,
    ParenCl,
    BracketOp,
    BracketCl,
    BraceOp,
    BraceCl,
    Semi,
    Comma,
    Dot,
    DotDotDot,
    At,
    ColonSep,

    // Variants of Java-*Operator*
    // =   >   <   !   ~   ?   :   ->
    Eq,
    Gt,
    Lt,
    Bang,
    Tilde,
    Question,
    Colon,
    Arrow,

    // ==  >=  <=  !=  &&  ||  ++  --
    EqEq,
    Ge,
    Le,
    Ne,
    AndAnd,
    OrOr,
    PlusPlus,
    MinusMinus,

    // +   -   *   /   &   |   ^   %   <<   >>   >>>
    Plus,
    Minus,
    Star,
    Slash,
    And,
    Or,
    Caret,
    Percent,
    Shl,
    Shr,
    ShrUn,

    // +=  -=  *=  /=  &=  |=  ^=  %=  <<=  >>=  >>>=
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    AndEq,
    OrEq,
    CaretEq,
    PercentEq,
    ShlEq,
    ShrEq,
    ShrUnEq,
}

impl Token {
    /// Returns true if the token is a "real" token (aka. a Java-*Token*)
    pub fn is_real(&self) -> bool {
        match *self {
            Token::Whitespace | Token::Comment => false,
            _ => true,
        }
    }

    /// String for error reporting. Example:
    /// ```
    /// Excpected one of `,` `;` `)`
    /// ```
    pub fn as_java_string(&self) -> &'static str {
        use self::Token::*;
        match self.clone() {
            Whitespace => "whitespace",
            Comment => "comment",

            Ident(_) => "identifier",
            Keyword(keyword) => keyword.as_java_string(),
            Literal(..) => "Lit(???)",

            ParenOp => "(",
            ParenCl => ")",
            BracketOp => "[",
            BracketCl => "]",
            BraceOp => "{",
            BraceCl => "}",
            Semi => ";",
            Comma => ",",
            Dot => ".",
            DotDotDot => "...",
            At => "@",
            ColonSep => "::",

            Eq => "=",
            Gt => ">",
            Lt => "<",
            Bang => "!",
            Tilde => "~",
            Question => "?",
            Colon => ":",
            Arrow => "->",

            EqEq => "==",
            Ge => ">=",
            Le => "<=",
            Ne => "!=",
            AndAnd => "&&",
            OrOr => "||",
            PlusPlus => "++",
            MinusMinus => "--",

            Plus => "+",
            Minus => "-",
            Star => "*",
            Slash => "/",
            And => "&",
            Or => "|",
            Caret => "^",
            Percent => "%",
            Shl => "<<",
            Shr => ">>",
            ShrUn => ">>>",

            PlusEq => "+=",
            MinusEq => "-=",
            StarEq => "*=",
            SlashEq => "/=",
            AndEq => "&=",
            OrEq => "|=",
            CaretEq => "^=",
            PercentEq => "%=",
            ShlEq => "<<=",
            ShrEq => ">>=",
            ShrUnEq => ">>>=",
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.as_java_string())
    }
}


// Macro to reduce repeated code for keywords.
macro_rules! declare_keywords {(
    $( ($name:ident, $word:expr); )*
) => {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum Keyword {
        $( $name, )*
    }

    impl Keyword {
        /// Returns the java string of the keyword
        pub fn as_java_string(&self) -> &'static str {
            match *self {
                $( Keyword::$name => $word, )*
            }
        }

        /// Returns the enum variant corresponding to the given string
        /// or None if the string does no represent a valid keyword.
        pub fn from_str(s: &String) -> Option<Self> {
            match &**s {
                $( $word => Some(Keyword::$name), )*
                _ => None,
            }
        }
    }

    impl Display for Keyword {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
            write!(f, "{}", self.as_java_string())
        }
    }
}}

declare_keywords! {
    // modifier
    (Public,        "public");
    (Protected,     "protected");
    (Private,       "private");
    (Abstract , "abstract");
    (Static , "static");
    (Final , "final");
    (Synchronized , "synchronized");
    (Native , "native");
    (Strictfp , "strictfp");
    (Transient , "transient");
    (Volatile , "volatile");

    (Class,         "class");
    (Import,        "import");

    // control structures
    (Do,     "do");
    (While,  "while");
    (For,    "for");
    (If,     "if");
    (Else,   "else");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    Str(String),
    /// Raw number, long suffix, radix
    Integer(String, bool, u8)
}
