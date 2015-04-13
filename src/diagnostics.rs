use filemap::FileMap;
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

    fn mark_line(&self, line_idx: u64, span: (u64, u64)) {
        let pre = format!("{}:{}", self.fmap.filename, line_idx);

        println!("{}| {}", pre, self.fmap.get_line(line_idx));

        // spaces(pre.len());
        print!("{pipe:>pipepad$} {start:>startpad$}",
            pipe="|", pipepad=pre.len() + 1,
            start=" ", startpad=span.0 as usize);

        colored!(YELLOW, println!("{0:-<1$}", "^", (span.1-span.0+2) as usize));

        // for _ in 0..prelen { print!(" "); }
        // colored!(t, BLUE, print!("| "));
        // colored!(t, YELLOW, {
        //     for _ in 0..startloc.col { print!(" "); }
        //     print!("^");
        //     for _ in startloc.col .. endloc.col + 1 { print!("-"); }
        // });
    }

    pub fn error_span(&self, span: (u64, u64), m: &str) {
        let startloc = self.fmap.get_loc(span.0);
        let endloc = self.fmap.get_loc(span.1 - 1);

        println!("");
        print!("{}:{}:{} .. {}:{}: ", self.fmap.filename,
            startloc.line, startloc.col,
            endloc.line, endloc.col);
        colored!(RED, print!("error: "));
        attrib!(Bold, println!("{}", m));

        self.mark_line(startloc.line, (startloc.col, endloc.col));

    }
}
