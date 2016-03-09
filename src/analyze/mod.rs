use syntax::{ast};
use base::{diag, code};
use syntax::ast::ItemExt;

pub fn check_names(ast: &ast::CompilationUnit, file: &code::FileMap) {
    for ty in &ast.types {
        if let Some(ref ident) = ty.ident() {
            check_upper_camel_case(ident, "type name", file);
        }
    }

}

fn check_upper_camel_case(name: &ast::Ident, context: &str, file: &code::FileMap) {
    // TODO unwrap
    let first_is_lowercase = name.name.chars().next().unwrap().is_lowercase();
    let has_underscores = name.name.find('_').is_some();
    if first_is_lowercase || has_underscores {
        let mut needs_uppercase = true;
        let new_name: String = name.name.chars().filter_map(|c| {
            if c == '_' {
                needs_uppercase = true;
                None
            } else if c.is_lowercase() && needs_uppercase {
                let up: Vec<_> = c.to_uppercase().collect();
                needs_uppercase = false;
                if up.len() == 1 {
                    Some(up[0])
                } else {
                    Some(c)
                }
            } else {
                Some(c)
            }
        }).collect();

        let rep = diag::Report::simple_warning(
            format!("This {} should be written in 'UpperCamelCase'", context),
            name.span,
        ).with_remark(diag::Remark::note(
            "Consider changing the name like shown here",
            diag::Snippet::Replace {
                span: name.span,
                with: new_name,
            }
        ));
        diag::print(&rep, &file, diag::PrintOptions::default());
    }
}
