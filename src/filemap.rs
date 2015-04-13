use std::io;
use std::rc::Rc;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::borrow::BorrowMut;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Loc {
    pub line: u64,
    pub col: u64,
}

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

    fn get_line_idx(&self, offset: u64) -> u64 {
        let l = self.lines.borrow();
        let mut a = 0;
        let mut z = l.len();
        while a < z - 1 {
            // println!("{}, {}", a, z);
            let m = (a+z)/2;

            // if the mth line starts after the offset, we have to search in
            // the first half.
            if (*l)[m] > offset {
                z = m;
            } else {
                a = m;
            }
        }
        (a + 2) as u64
    }

    pub fn get_loc(&self, offset: u64) -> Loc {
        let line = self.get_line_idx(offset);
        let col = offset - (*self.lines.borrow())[(line - 2) as usize] - 1;

        Loc { line: line, col: col }
    }

    pub fn get_line(&self, index: u64) -> String {
        // TODO: Maybe safety checks
        let offset = self.lines.borrow()[index as usize - 2] as usize - 1;
        let end = self.src[offset .. self.src.len()]
            .find("\n").unwrap_or(self.src.len());
        self.src[offset .. (end + offset)].to_string()
    }
}

pub fn open_file(pa: &Path) -> io::Result<Rc<FileMap>> {
    let mut file = try!(File::open(pa));
    let mut s = String::new();
    try!(file.read_to_string(&mut s));

    let fname = pa.file_name().unwrap().to_str().unwrap().to_string();
    Ok(Rc::new(FileMap::new(fname, s)))
}
