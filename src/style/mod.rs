use syntax::ast;
use diagnostics::ErrorHandler;
use std::default::Default;
use std::fmt::{Display, Formatter, Error};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NameType {
    Class,
    Method,
    StaticMethod,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NameStyle {
    LowerCamelCase, // lowerCamelCase
    UpperCamelCase, // UpperCamelCase
    SnakeCase,      // snake_case
    UpperSnakeCase, // Upper_Snake_Case
    CapsSnakeCase,  // CAPS_SNAKE_CASE
}

impl Display for NameStyle {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", match *self {
            NameStyle::LowerCamelCase => "lowerCamelCase",
            NameStyle::UpperCamelCase => "UpperCamelCase",
            NameStyle::SnakeCase => "snake_case",
            NameStyle::UpperSnakeCase => "Upper_Snake_Case",
            NameStyle::CapsSnakeCase => "CAPS_SNAKE_CASE",
        })
    }
}


#[allow(dead_code)]
pub struct Config {
    class_s: NameStyle,
    method_s: NameStyle,
    static_method_s: NameStyle,
}

impl Config {
    fn get(&self, ty: NameType) -> NameStyle {
        match ty {
            NameType::Class => self.class_s,
            NameType::Method => self.method_s,
            NameType::StaticMethod => self.static_method_s,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            class_s: NameStyle::UpperCamelCase,
            method_s: NameStyle::LowerCamelCase,
            static_method_s: NameStyle::LowerCamelCase,
        }
    }
}

pub struct Checker<'a> {
    e: &'a ErrorHandler,
    conf: &'a Config,
}

impl<'a> Checker<'a> {
    pub fn new(e: &'a ErrorHandler, c: &'a Config) -> Checker<'a> {
        Checker { e: e, conf: c }
    }

    pub fn check(&self, cu: &ast::CUnit) {
        // check class name
        for item in &cu.items {
            match *item {
                ast::Item::Class(ref c) => { self.check_class(c) },
                _ => {},
            }
        }
    }

    fn check_class(&self, c: &ast::Class) {
        self.check_ident(&c.name, NameType::Class);

        for m in &c.methods {
            let name_ty = if m.static_ {
                NameType::StaticMethod
            } else {
                NameType::Method
            };
            self.check_ident(&m.name, name_ty);
        }
    }

    fn check_ident(&self, id: &ast::Ident, ty: NameType) {
        self.e.span_note(id.span, format!(
            "This is a '{:?}' name and needs to be '{}' :-)",
            ty, self.conf.get(ty)
        ));
    }
}
