use std::vec::Vec;

// A Java compilation unit: "File that contains one class"
#[derive(Debug, Clone)]
pub struct CUnit {
    pub imports: Vec<Import>,
    pub class: Class,
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
pub struct Class;

#[derive(Debug, Clone)]
pub struct Name {
    // for qualified names
    pub path: Vec<String>,
    pub last: Option<String>,
}
