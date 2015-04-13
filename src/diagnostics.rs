use filemap::{FileMap, Span, Loc};
use std::rc::Rc;
use term;

pub struct ErrorHandler {
    fmap: Rc<FileMap>,
}

macro_rules! colored {
    ($c:ident, $p:expr ) => ({
        term::stdout().unwrap().fg(term::color::$c).unwrap();
        $p;
        term::stdout().unwrap().reset().unwrap();
    })
}

macro_rules! attrib {
    ($a:ident, $p:expr ) => ({
        term::stdout().unwrap().attr(term::Attr::$a).unwrap();
        $p;
        term::stdout().unwrap().reset().unwrap();
    })
}

#[allow(dead_code)]
impl ErrorHandler {
    pub fn new(fmap: Rc<FileMap>) -> ErrorHandler {
        ErrorHandler { fmap: fmap }
    }

    pub fn error(&self, m: &str) {
        println!("");
        print!("{}: ", self.fmap.filename);
        colored!(RED, print!("error: "));
        attrib!(Bold, println!("{}", m));
    }

    fn print_snippet(&self, start: Loc, end: Loc) {
        if start.line == end.line {
            let pre = format!("{}:{}: ", self.fmap.filename, start.line);

            println!("{}{}", pre, self.fmap.get_line(start.line));

            // Print spaces until the span start is reached
            print!("{0:>1$}", " ", pre.len() + start.col);

            colored!(YELLOW, println!("{0:-<1$}", "^", (end.col-start.col+2)));
        } else {
            for line in start.line .. end.line + 1 {
                let pre = format!("{}:{}: ", self.fmap.filename, line);
                println!("{}{}", pre, self.fmap.get_line(line));
            }
        }
    }

    pub fn error_span(&self, span: Span, m: &str) {
        let start = self.fmap.get_loc(span.lo);
        let end = self.fmap.get_loc(span.hi - 1);

        println!("");
        print!("{}:{}:{} .. {}:{}: ", self.fmap.filename,
            start.line, start.col,
            end.line, end.col);
        colored!(RED, print!("error: "));
        attrib!(Bold, println!("{}", m));

        self.print_snippet(start, end);

    }
}
