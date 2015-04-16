use filemap::{FileMap, Span, Loc};
use std::rc::Rc;
// use std::iter::repeat;
use term_painter::ToStyle;
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

    fn print_snippet(&self, start: Loc, end: Loc) {
        if start.line == end.line {
            let pre = format!("{}:{}: ", self.fmap.filename, start.line);

            println!("{}:{}: {}",
                self.fmap.filename,
                Blue.paint(start.line),
                self.fmap.get_line(start.line));

            // Print spaces until the span start is reached
            print!("{0:>1$}", " ", pre.len() + start.col);

            Yellow.with(|| {
                println!("{:-<1$}", "^", end.col-start.col+1);
            });

        } else {
            for line in start.line .. end.line + 1 {
                println!("{}:{}: {}",
                    self.fmap.filename,
                    Blue.paint(line),
                    self.fmap.get_line(line));
            }
        }
    }

    pub fn span_err(&self, span: Span, m: &str) {
        let start = self.fmap.get_loc(span.lo);
        let end = self.fmap.get_loc(span.hi - 1);

        println!("");
        println!("{file}:{sl}:{sc} .. {el}:{ec}: {error} {m}",
            file=self.fmap.filename,
            sl=Blue.paint(start.line), sc=start.col,
            el=Blue.paint(end.line), ec=end.col,
            error=Red.paint("error:"), m=Bold.paint(m));

        self.print_snippet(start, end);

    }
}
