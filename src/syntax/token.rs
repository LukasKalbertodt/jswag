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
        // pub fn word(&self) -> &'static str {
        //     match *self {
        //         $( Keyword::$name => $word, )*
        //     }
        // }

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
    (Public,        "public");
    (Private,       "private");
    (Protected,     "protected");
    (Class,         "class");
    (Static,        "static");
    (Import,        "import");

    // control structures
    (Do,     "do");
    (While,  "while");
    (For,    "for");
    (If,     "if");
    (Else,   "else");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelimToken {
    Paren,      // round ( )
    Bracket,    // square [ ]
    Brace,      // curly { }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    Str(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Ignored tokens
    Whitespace(bool),   // true if it contains a newline
    Comment,

    // Simple one char tokens
    Dot,
    Comma,
    Semi,

    // Long string tokens
    Keyword(Keyword),
    Word(String),
    Other(String),

    Literal(Lit),

    OpenDelim(DelimToken),
    CloseDelim(DelimToken),
}

impl Token {
    pub fn is_real(&self) -> bool {
        match *self {
            Token::Whitespace(false) | Token::Comment => false,
            _ => true,
        }
    }
}
