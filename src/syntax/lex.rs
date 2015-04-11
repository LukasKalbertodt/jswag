
use std::str::Chars;
use std::iter::Iterator;

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


// pub enum TokenType {
//     Keyword(Keyword),
//     Other(String),
// }

// pub struct Token {
//     pub ty: TokenType,
//     pub span: (i64, i64),
// }

pub struct Tokenizer<'a> {
    chs: Chars<'a>,

    curr: Option<char>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(content: &'a String) -> Tokenizer<'a> {
        let iter = content.chars();
        let mut tok = Tokenizer {
            chs: iter,
            curr: None,
        };
        tok.bump();
        tok
    }

    fn bump(&mut self) {
        let ch = self.chs.next();
        self.curr = ch;
    }

    fn skip_whitespace(&mut self) {
        while self.curr.is_some() && is_whitespace(self.curr.unwrap()) {
            self.bump();
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.skip_whitespace();

        if self.curr.is_none() {
            return None;
        }

        let mut s = String::new();
        while self.curr.is_some() && !is_whitespace(self.curr.unwrap()) {
            s.push(self.curr.unwrap());
            self.bump();
        }

        // let last = self.curr;
        // self.bump();
        // last

        Some(s)


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
