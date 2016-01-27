// TODO: maybe write a macro to generate these macros

/// Helper to do something conditionally depending on the `--verbose` flag
macro_rules! verbose {
    ($body:block) => {{
        if $crate::VERBOSE.load(::std::sync::atomic::Ordering::Relaxed) {
            $body
        }
    }}
}

macro_rules! error {
    ($fmt:expr) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Color::Red,
                "!! error:"
            )
        );
    );
    ($fmt:expr, $($args:tt)*) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Color::Red,
                "!! error:"
            ),
            $($args)*
        )
    );
}

macro_rules! warning {
    ($fmt:expr) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Color::Yellow,
                "   warning:"
            )
        );
    );
    ($fmt:expr, $($args:tt)*) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Color::Yellow,
                "   warning:"
            ),
            $($args)*
        )
    );
}

macro_rules! note {
    ($fmt:expr) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Color::Green,
                "   note:"
            )
        );
    );
    ($fmt:expr, $($args:tt)*) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Color::Green,
                "   note:"
            ),
            $($args)*
        )
    );
}

macro_rules! executing {
    ($fmt:expr) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Attr::Bold,
                " -- executing:"
            )
        );
    );
    ($fmt:expr, $($args:tt)*) => (
        println!(
            concat!("{} ", $fmt),
            ::term_painter::ToStyle::paint(
                &::term_painter::Attr::Bold,
                " -- executing:"
            ),
            $($args)*
        )
    );
}
