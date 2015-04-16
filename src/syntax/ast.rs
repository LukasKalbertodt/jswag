use std::vec::Vec;

// A Java compilation unit: "File that contains one class"
#[derive(Debug, Clone)]
pub struct CUnit {
    pub imports: Vec<Import>,
    pub class: Class,
}

// A import declaration
#[derive(Debug, Copy, Clone)]
pub enum Import {
    // import IO.AlgoTools;
    SingleType,
    // called "type-import-on-demand" in specs
    // import IO.*;
    Wildcard,
}

#[derive(Debug, Copy, Clone)]
pub struct Class;
