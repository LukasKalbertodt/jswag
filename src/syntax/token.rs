/// Module `token`:
/// Contains enums and structs, that describe token types in Java.

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
pub enum BinOpToken {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    And,
    Or,
    Shl,
    Shr,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DelimToken {
    Paren,      // round ( )
    Bracket,    // square [ ]
    Brace,      // curly { }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    Str(String),
    Integer(String)
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Ignored tokens
    Whitespace,
    Comment,

    // Simple one char tokens
    Dot,
    Comma,
    Semi,

    // Operators
    Eq,
    Lt,
    Le,
    EqEq,
    Ne,
    Ge,
    Gt,
    AndAnd,
    OrOr,
    Not,
    Tilde,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    And,
    Or,
    Shl,
    Shr,

    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    CaretEq,
    AndEq,
    OrEq,
    ShlEq,
    ShrEq,

    // Long string tokens
    Keyword(Keyword),
    Word(String),

    Literal(Lit),

    OpenDelim(DelimToken),
    CloseDelim(DelimToken),
}

impl Token {
    // Returns true if the token is not an ignored token (whitespace/comment)
    pub fn is_real(&self) -> bool {
        match *self {
            Token::Whitespace | Token::Comment => false,
            _ => true,
        }
    }

    pub fn as_java_string(&self) -> String {
        match self.clone() {
            Token::Whitespace => "'whitespace'",
            Token::Comment => "'comment'",

            Token::Dot => ".",
            Token::Comma => ",",
            Token::Semi => ";",

            Token::Eq => "=",
            Token::Lt => "<",
            Token::Le => "<=",
            Token::EqEq => "==",
            Token::Ne => "!=",
            Token::Ge => ">=",
            Token::Gt => ">",
            Token::AndAnd => "&&",
            Token::OrOr => "||",
            Token::Not => "!",
            Token::Tilde => "~",

            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Percent => "%",
            Token::Caret => "^",
            Token::And => "&",
            Token::Or => "|",
            Token::Shl => "<<",
            Token::Shr => ">>",

            Token::PlusEq => "+=",
            Token::MinusEq => "-=",
            Token::StarEq => "*=",
            Token::SlashEq => "/=",
            Token::PercentEq => "%=",
            Token::CaretEq => "^=",
            Token::AndEq => "&=",
            Token::OrEq => "|=",
            Token::ShlEq => "<<=",
            Token::ShrEq => ">>=",

            Token::Keyword(keyword) => keyword.as_java_string(),
            // Token::Word(ref w) => format!("Word('{}')", w).as_str() ,
            Token::Word(ref w) => {
                if w.is_empty() {
                    "Ident"
                } else {
                    w.as_ref()
                }
            },

            Token::Literal(..) => "Lit(???)",

            Token::OpenDelim(delim) => match delim {
                DelimToken::Brace => "{",
                DelimToken::Paren => "(",
                DelimToken::Bracket => "[",
            },
            Token::CloseDelim(delim) => match delim {
                DelimToken::Brace => "}",
                DelimToken::Paren => ")",
                DelimToken::Bracket => "]",
            },

            // _ => "'???'",
        }.to_string()
    }
}
