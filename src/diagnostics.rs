use filemap::{FileMap, Span, SrcIndex};
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

impl ErrorHandler {
    pub fn new(fmap: Rc<FileMap>) -> ErrorHandler {
        ErrorHandler { fmap: fmap }
    }

    // pub fn error(&self, m: &str) {
    //     println!("");
    //     print!("{}: ", self.fmap.filename);
    //     colored!(RED, print!("error: "));
    //     attrib!(Bold, println!("{}", m));
    // }

    fn mark_line(&self, line_idx: SrcIndex, span: Span) {
        let pre = format!("{}:{}", self.fmap.filename, line_idx);

        println!("{}: {}", pre, self.fmap.get_line(line_idx));

        // spaces(pre.len());
        print!("{pre:>prepad$} {start:>startpad$}",
            pre=" ", prepad=pre.len() + 1,
            start=" ", startpad=span.lo as usize);

        colored!(YELLOW, println!("{0:-<1$}", "^", (span.hi-span.lo+2) as usize));
    }

    pub fn error_span(&self, span: Span, m: &str) {
        let startloc = self.fmap.get_loc(span.lo);
        let endloc = self.fmap.get_loc(span.hi - 1);

        println!("");
        print!("{}:{}:{} .. {}:{}: ", self.fmap.filename,
            startloc.line, startloc.col,
            endloc.line, endloc.col);
        colored!(RED, print!("error: "));
        attrib!(Bold, println!("{}", m));

        self.mark_line(startloc.line, Span { lo: startloc.col, hi: endloc.col });

    }
}
