use filemap::{FileMap, Span, Loc};
use std::rc::Rc;
// use std::iter::repeat;
use term_painter::{ToStyle, Color};
use term_painter::Color::*;
use term_painter::Attr::*;

pub struct ErrorHandler {
    fmap: Rc<FileMap>,
}

// fn times(s: &str, rhs: usize) -> String {
//     repeat(s).take(rhs).collect()
// }

#[allow(dead_code)]
impl ErrorHandler {
    pub fn new(fmap: Rc<FileMap>) -> ErrorHandler {
        ErrorHandler { fmap: fmap }
    }

    pub fn err(&self, m: &str) {
        println!("");
        print!("{}: {} {}",
            self.fmap.filename, Red.paint("error:"), Bold.paint(m));
    }

    fn print_snippet(&self, start: Loc, end: Loc, ucol: Color) {
        if start.line == end.line {
            let pre = format!("{}:{}: ", self.fmap.filename, start.line);

            println!("{}:{}: {}",
                self.fmap.filename,
                Blue.paint(start.line + 1),
                self.fmap.get_line(start.line));

            // Print spaces until the span start is reached
            print!("{0:>1$}", " ", pre.len() + start.col);

            let uline_len = 1 + if end.col < start.col {
                start.col - end.col
            } else {
                end.col-start.col
            };

            ucol.with(|| {
                println!("{:-<1$}", "^", uline_len);
            });

        } else {
            for line in start.line .. end.line + 1 {
                println!("{}:{}: {}",
                    self.fmap.filename,
                    Blue.paint(line + 1),
                    self.fmap.get_line(line));
            }
        }
    }

    pub fn span_err(&self, span: Span, m: String) {
        let start = self.fmap.get_loc(span.lo);
        let end = self.fmap.get_loc(span.hi);

        println!("");
        println!("{file}:{sl}:{sc} .. {el}:{ec}: {error} {m}",
            file=self.fmap.filename,
            sl=Blue.paint(start.line + 1), sc=start.col,
            el=Blue.paint(end.line + 1), ec=end.col,
            error=Red.paint("error:"), m=Bold.paint(m));

        self.print_snippet(start, end, Red);

    }

    pub fn span_note(&self, span: Span, m: String) {
        let start = self.fmap.get_loc(span.lo);
        let end = self.fmap.get_loc(span.hi);

        println!("");
        println!("{file}:{sl}:{sc} .. {el}:{ec}: {error} {m}",
            file=self.fmap.filename,
            sl=Blue.paint(start.line + 1), sc=start.col,
            el=Blue.paint(end.line + 1), ec=end.col,
            error=Green.paint("note:"), m=Bold.paint(m));

        self.print_snippet(start, end, Green);
    }
}
