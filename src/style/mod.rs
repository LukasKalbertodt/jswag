use syntax::ast;
use diagnostics::ErrorHandler;

pub struct Checker<'a> {
    e: &'a ErrorHandler,
}

impl<'a> Checker<'a> {
    pub fn new(e: &'a ErrorHandler) -> Checker<'a> {
        Checker { e: e }
    }

    pub fn check(&self, cu: &ast::CUnit) {
        // check class name
        for item in &cu.items {
            match *item {
                ast::Item::Class(ref c) => {
                    self.check_ident(&c.name);
                }
                _ => {},
            }
        }
    }

    fn check_ident(&self, id: &ast::Ident) {
        self.e.span_err(id.span, format!("This is a name :-)"));
    }
}
