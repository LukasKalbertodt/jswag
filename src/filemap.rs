use std::io;
use std::rc::Rc;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::borrow::BorrowMut;
use std::cell::RefCell;

// pub struct Loc {
//     pub line: u64,
//     pub col: u64,
// }

pub struct FileMap {
    pub filename: String,
    pub src: String,
    lines: RefCell<Vec<u64>>,
}

impl FileMap {
    fn new(filename: String, src: String) -> FileMap {
        FileMap {
            filename: filename,
            src: src,
            lines: RefCell::new(Vec::new()),
        }
    }

    pub fn add_line(&self, offset: u64) {
        let mut lines = self.lines.borrow_mut();
        lines.push(offset);
    }

    pub fn num_lines(&self) -> u64 {
        self.lines.borrow().len() as u64
    }
}

pub fn open_file(pa: &Path) -> io::Result<Rc<FileMap>> {
    let mut file = try!(File::open(pa));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));

    let fname = pa.file_name().unwrap().to_str().unwrap().to_string();
    Ok(Rc::new(FileMap::new(fname, s)))
}
