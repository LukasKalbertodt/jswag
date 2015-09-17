// TODO: Remove
#![allow(dead_code)]

use std::fmt::{Display, Formatter, Error};
use filemap::Span;
use std::vec::Vec;
use std::default::Default;

macro_rules! java_enum { (
    $name:ident { $( $variant:ident => $java_word:expr, )* }
) => {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
    pub enum $name {
        $( $variant, )*
    }

    impl $name {
        pub fn as_java_string(&self) -> &str {
            match *self {
                $( $name::$variant => $java_word , )*
            }
        }
    }

    impl Display for $name {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
            write!(f, "{}", self.as_java_string())
        }
    }
}}


// A Java compilation unit: "File that contains one class"
#[derive(Debug, Clone)]
pub struct CUnit {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Import(Import),
    Class(Box<Class>),
    Method(Box<Method>),
}


#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Default for Ident {
    fn default() -> Self {
        Ident {
            name: "".to_string(),
            span: Span { lo: 0, hi: 0 },
        }
    }
}

// A import declaration
#[derive(Debug, Clone)]
pub enum Import {
    // import IO.AlgoTools;
    Single(Name),
    // called "type-import-on-demand" in specs
    // import IO.*;
    Wildcard(Name),
}

#[derive(Debug, Copy, Clone)]
pub enum Visibility {
    Public,
    Protected,
    Package,
    Private,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Ident,
    pub vis: Visibility,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone)]
pub struct Name {
    // for qualified names
    pub path: Vec<Ident>,
    pub last: Option<Ident>,
}

java_enum! (Modifier {
    Public => "public",
    Protected => "protected",
    Private => "private",
    Abstract => "abstract",
    Static => "static",
    Final => "final",
    Synchronized => "synchronized",
    Native => "native",
    Strictfp => "strictfp",
    Transient => "transient",
    Volatile => "volatile",
});

#[derive(Debug, Clone)]
pub struct Method {
    pub vis: Visibility,
    pub name: Ident,
    pub ret_ty: Ident,
    pub static_: bool,
    pub final_: bool,
    pub params: Vec<FormalParameter>,
}

#[derive(Debug, Clone)]
pub struct FormalParameter {
    pub ty: Ident,
    pub name: Ident,
    pub dims: usize,
    pub final_: bool,
}
