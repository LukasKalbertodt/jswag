use term_painter::ToStyle;
use term_painter::Color::*;
use std::fmt;

pub const MAX_MESSAGE_LEN: usize = 9;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum MessageType {
    // generic messages
    Error,
    Warning,

    // action
    Compiling,
    Running,
    Ignoring,
    Aborting,

    // status
    Fresh,

    Note,
    Debug,
    None,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status_style = Green.bold();

        let (msg, style) = match *self {
            MessageType::Error => ("Error", Red.bold()),
            MessageType::Warning => ("Warning", Yellow.to_style()),
            MessageType::Aborting => ("Aborting", Magenta.bold()),
            MessageType::Compiling => ("Compiling", status_style),
            MessageType::Running => ("Running", status_style),
            MessageType::Ignoring => ("Ignoring", White.bold()),
            MessageType::Fresh => ("Fresh", status_style),
            MessageType::Note => ("Note", White.bold()),
            MessageType::Debug => ("Debug", NotSet.to_style()),
            MessageType::None => {
                return write!(f, "{:>1$}", " ", MAX_MESSAGE_LEN + 2);
            },
        };

        write!(f, "{:>1$}", style.paint(msg), MAX_MESSAGE_LEN + 2)
    }
}

macro_rules! msg {
    ($ty:ident, $fmt:expr) => {
        println!(
            concat!("{} | ", $fmt),
            $crate::ui::MessageType::$ty,
        );
    };
    ($ty:ident, $fmt:expr, $($args:tt)*) => {
        println!(
            concat!("{} | ", $fmt),
            $crate::ui::MessageType::$ty,
            $($args)*
        )
    };
}
