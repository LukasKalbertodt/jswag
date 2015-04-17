use std::vec::Vec;

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
}}


// A Java compilation unit: "File that contains one class"
#[derive(Debug, Clone)]
pub struct CUnit {
    pub imports: Vec<Import>,
    pub class: Option<TopLevelClass>,
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
pub struct TopLevelClass {
    // Note: Just Public and Package is valid for TopLevelClasses
    pub vis: Visibility,
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone)]
pub struct Name {
    // for qualified names
    pub path: Vec<String>,
    pub last: Option<String>,
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
    pub name: String,
    pub return_ty: String,
}
