use std::io;
use std::rc::Rc;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::borrow::BorrowMut;
use std::cell::RefCell;

pub type SrcIndex = usize;
pub type LineIdx = usize;

#[derive(Debug)]
pub struct Loc {
    pub line: SrcIndex,
    pub col: SrcIndex,
}


/// Span in the source string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    /// Inclusive
    pub lo: SrcIndex,
    /// Exclusive
    pub hi: SrcIndex,
}



pub struct FileMap {
    pub filename: String,
    pub src: String,
    lines: RefCell<Vec<SrcIndex>>,
}

impl FileMap {
    pub fn new(filename: String, src: String) -> FileMap {
        FileMap {
            filename: filename,
            src: src,
            lines: RefCell::new(vec![0]),
        }
    }

    pub fn add_line(&self, offset: SrcIndex) {
        let mut lines = self.lines.borrow_mut();
        lines.push(offset);
    }

    // pub fn num_lines(&self) -> usize {
    //     self.lines.borrow().len()
    // }

    fn get_line_idx(&self, offset: SrcIndex) -> SrcIndex {
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
        a
    }

    pub fn get_loc(&self, offset: SrcIndex) -> Loc {
        let line = self.get_line_idx(offset);
        let col = offset - (*self.lines.borrow())[line] - 1;

        Loc { line: line, col: col }
    }

    pub fn get_line(&self, index: LineIdx) -> String {
        // TODO: Maybe safety checks
        let offset = self.lines.borrow()[index];
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
