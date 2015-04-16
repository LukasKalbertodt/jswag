use std::vec::Vec;

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
    pub visibility: Visibility,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Name {
    // for qualified names
    pub path: Vec<String>,
    pub last: Option<String>,
}
