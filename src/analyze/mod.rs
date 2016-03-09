// BEWARE: low quality shit code ahead
// TODO: make americode great again

use syntax::{ast};
use base::{diag, code};
use syntax::ast::ItemExt;

pub fn check_names(ast: &ast::CompilationUnit, file: &code::FileMap) {
    for ty in &ast.types {
        if let Some(ref ident) = ty.ident() {
            check_upper_camel_case(ident, "type name", file);
        }

        if let &ast::TypeDef::NormalClass(ref c) = ty {
            for member in &c.members {
                if let &ast::ClassMember::Method(ref method) = member {
                    check_lower_camel_case(&method.name, "method name", file);
                }
            }
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

fn check_lower_camel_case(name: &ast::Ident, context: &str, file: &code::FileMap) {
    // TODO unwrap
    let first_is_uppercase = name.name.chars().next().unwrap().is_uppercase();
    let has_underscores = name.name.find('_').is_some();
    if first_is_uppercase || has_underscores {
        let mut needs_uppercase = false;
        let mut new_name: String = name.name.chars().skip(1).filter_map(|c| {
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

        let first_lower: Vec<_> = name.name.chars().next().unwrap().to_lowercase().collect();
        let first_lower = if first_lower.len() == 1 {
            first_lower[0]
        } else {
            name.name.chars().next().unwrap()
        };
        new_name.insert(0, first_lower);

        let rep = diag::Report::simple_warning(
            format!("This {} should be written in 'lowerCamelCase'", context),
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
