use filemap::FileMap;
use std::rc::Rc;
use term;

pub struct ErrorHandler {
    file: Rc<FileMap>,
}

macro_rules! colored {
    ($t:ident, $c:ident, $p:expr ) => ({
        $t.fg(term::color::$c).unwrap();
        $p;
        $t.reset().unwrap();
    })
}

macro_rules! attrib {
    ($t:ident, $a:ident, $p:expr ) => ({
        $t.attr(term::Attr::$a).unwrap();
        $p;
        $t.reset().unwrap();
    })
}

impl ErrorHandler {
    pub fn new(fmap: Rc<FileMap>) -> ErrorHandler {
        ErrorHandler { file: fmap }
    }

    pub fn error(&self, m: &str) {
        let mut t = term::stdout().unwrap();

        println!("");
        print!("{}: ", self.file.filename);
        colored!(t, RED, print!("error: "));
        attrib!(t, Bold, println!("{}", m));
    }
}
