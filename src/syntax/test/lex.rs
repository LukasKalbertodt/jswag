#![allow(unused)]
/// This module contains unit tests for the tokenizer.
///

use syntax::*;
use syntax::token::Token::*;
use filemap::{FileMap, Span};
use std::rc::Rc;
use diagnostics::ErrorHandler;


fn spans(src: &str) -> Vec<TokenSpan> {
    let fmap = Rc::new(FileMap::new("<unit-test>".into(), src.into()));
    let error_handler = ErrorHandler::new(fmap.clone());
    let toks = Tokenizer::new(&fmap, &error_handler);
    toks.collect()
}


fn toks(src: &str) -> Vec<Token> {
    spans(src).into_iter().map(|ts| ts.tok).collect()
}

fn reals(src: &str) -> Vec<Token> {
    toks(src).into_iter().filter(|t| t.is_real()).collect()
}

macro_rules! toks {
    ($s:expr, [$($v:expr),*]) => {
        assert_eq!(toks($s), vec![$($v),*])
    }
}
macro_rules! reals {
    ($s:expr, [$($v:expr),*]) => {
        assert_eq!(reals($s), vec![$($v),*])
    }
}


#[test]
fn empty() {
    toks!("", []);
}

#[test]
fn idents() {
    toks!("foo", [Ident("foo".into())]);
    toks!("foo bar", [
        Ident("foo".into()),
        Whitespace,
        Ident("bar".into())
    ]);
    toks!("1bla", [
        Literal(Lit::Integer { raw: "1".into(), is_long: false, radix: 10 }),
        Ident("bla".into())
    ]);
    toks!("b1la", [Ident("b1la".into())]);

}

#[test]
fn ops() {
    // all seperators and operators
    reals!("(   )   {   }   [   ]   ;   ,   .   ...   @   ::", [
        ParenOp, ParenCl, BraceOp, BraceCl, BracketOp, BracketCl,
        Semi, Comma, Dot, DotDotDot, At, ColonSep
    ]);
    reals!("=   >   <   !   ~   ?   :   ->", [
        Eq, Gt, Lt, Bang, Tilde, Question, Colon, Arrow
    ]);
    reals!("==  >=  <=  !=  &&  ||  ++  --", [
        EqEq, Ge, Le, Ne, AndAnd, OrOr, PlusPlus, MinusMinus
    ]);
    reals!("+   -   *   /   &   |   ^   %   <<   >>   >>>", [
        Plus, Minus, Star, Slash, And, Or, Caret, Percent, Shl, Shr, ShrUn
    ]);
    reals!("+=  -=  *=  /=  &=  |=  ^=  %=  <<=  >>=  >>>=", [
        PlusEq, MinusEq, StarEq, SlashEq, AndEq, OrEq, CaretEq, PercentEq,
        ShlEq, ShrEq, ShrUnEq
    ]);

    // multi char op stress test
    reals!(">>>>>>=>> >>=> >=", [ShrUn, ShrUnEq, Shr, ShrEq, Gt, Ge]);
    reals!("<< <<=< <=", [Shl, ShlEq, Lt, Le]);
}

#[test]
fn easy_literals() {
    reals!("true false null", [
        Literal(Lit::Bool(true)), Literal(Lit::Bool(false)), Literal(Lit::Null)
    ]);
    reals!("truefalse null", [Ident("truefalse".into()), Literal(Lit::Null)]);
}

#[test]
fn int_literals() {
    assert_eq!(toks("123"), vec![
        Literal(Lit::Integer { raw: "123".into(), is_long: false, radix: 10 })
    ]);
    assert_eq!(toks("123l"), vec![
        Literal(Lit::Integer { raw: "123".into(), is_long: true, radix: 10 })
    ]);
    assert_eq!(toks("0123"), vec![
        Literal(Lit::Integer { raw: "123".into(), is_long: false, radix: 8 })
    ]);
    assert_eq!(toks("0x1fa3l"), vec![
        Literal(Lit::Integer { raw: "1fa3".into(), is_long: true, radix: 16 })
    ]);
    assert_eq!(toks("0x1f"), vec![
        Literal(Lit::Integer { raw: "1f".into(), is_long: false, radix: 16 })
    ]);
    assert_eq!(toks("0b101l"), vec![
        Literal(Lit::Integer { raw: "101".into(), is_long: true, radix: 2 })
    ]);
    assert_eq!(toks("0l"), vec![
        Literal(Lit::Integer { raw: "0".into(), is_long: true, radix: 10 })
    ]);
}

#[test]
fn float_literals() {
    // type 1:  Digits . [Digits] [ExponentPart] [FloatTypeSuffix]
    toks!("3.", [Literal(Lit::Float {   // digit dot
        raw: "3.".into(),
        is_double: true,
        radix: 10,
        exp: "".into()
    })]);
    toks!("3.14", [Literal(Lit::Float { // digit dot digit
        raw: "3.14".into(),
        is_double: true,
        radix: 10,
        exp: "".into()
    })]);
    toks!("3.e2", [Literal(Lit::Float { // digit dot exp
        raw: "3.".into(),
        is_double: true,
        radix: 10,
        exp: "2".into()
    })]);
    toks!("3.f", [Literal(Lit::Float {  // digit dot suffix
        raw: "3.".into(),
        is_double: false,
        radix: 10,
        exp: "".into()
    })]);
    toks!("3.14e-3", [Literal(Lit::Float {   // digit dot digit exp
        raw: "3.14".into(),
        is_double: true,
        radix: 10,
        exp: "-3".into()
    })]);
    toks!("3.14f", [Literal(Lit::Float {    // digit dot digit suffix
        raw: "3.14".into(),
        is_double: false,
        radix: 10,
        exp: "".into()
    })]);
    toks!("3.e3f", [Literal(Lit::Float {    // digit dot exp suffix
        raw: "3.".into(),
        is_double: false,
        radix: 10,
        exp: "3".into()
    })]);
    toks!("3.14e-3f", [Literal(Lit::Float {  // digit dot digit exp suffix
        raw: "3.14".into(),
        is_double: false,
        radix: 10,
        exp: "-3".into()
    })]);

    // type 2: . Digits [ExponentPart] [FloatTypeSuffix]
    toks!(".14", [Literal(Lit::Float {
        raw: ".14".into(),
        is_double: true,
        radix: 10,
        exp: "".into()
    })]);
    toks!(".14e-3", [Literal(Lit::Float {
        raw: ".14".into(),
        is_double: true,
        radix: 10,
        exp: "-3".into()
    })]);
    toks!(".14f", [Literal(Lit::Float {
        raw: ".14".into(),
        is_double: false,
        radix: 10,
        exp: "".into()
    })]);
    toks!(".14e3f", [Literal(Lit::Float {
        raw: ".14".into(),
        is_double: false,
        radix: 10,
        exp: "3".into()
    })]);

    // type 3:  Digits ExponentPart [FloatTypeSuffix]
    // type 4: Digits [ExponentPart] FloatTypeSuffix
    toks!("3e-3", [Literal(Lit::Float {
        raw: "3".into(),
        is_double: true,
        radix: 10,
        exp: "-3".into()
    })]);
    toks!("3f", [Literal(Lit::Float {
        raw: "3".into(),
        is_double: false,
        radix: 10,
        exp: "".into()
    })]);
    toks!("3e3f", [Literal(Lit::Float {
        raw: "3".into(),
        is_double: false,
        radix: 10,
        exp: "3".into()
    })]);


    // floating point literals
    toks!("0x3p4", [Literal(Lit::Float {
        raw: "3".into(),
        is_double: true,
        radix: 16,
        exp: "4".into()
    })]);
    toks!("0x3p4_4f", [Literal(Lit::Float {
        raw: "3".into(),
        is_double: false,
        radix: 16,
        exp: "44".into()
    })]);

    toks!("0x3.p4", [Literal(Lit::Float {
        raw: "3.".into(),
        is_double: true,
        radix: 16,
        exp: "4".into()
    })]);
    toks!("0x378.p4f", [Literal(Lit::Float {
        raw: "378.".into(),
        is_double: false,
        radix: 16,
        exp: "4".into()
    })]);

    toks!("0x3.1_55p43", [Literal(Lit::Float {
        raw: "3.155".into(),
        is_double: true,
        radix: 16,
        exp: "43".into()
    })]);
    toks!("0x3.1p4f", [Literal(Lit::Float {
        raw: "3.1".into(),
        is_double: false,
        radix: 16,
        exp: "4".into()
    })]);

    toks!("0x.11p4", [Literal(Lit::Float {
        raw: ".11".into(),
        is_double: true,
        radix: 16,
        exp: "4".into()
    })]);
    toks!("0x.1p44f", [Literal(Lit::Float {
        raw: ".1".into(),
        is_double: false,
        radix: 16,
        exp: "44".into()
    })]);
}

#[test]
fn string_literals() {
    toks!(r#""hi""#, [Literal(Lit::Str("hi".into()))]);
    toks!(r#""hi \" bla""#, [Literal(Lit::Str("hi \" bla".into()))]);
    toks!(r#""\b \t \n \f \r \" \' \\""#, [Literal(Lit::Str(
        "\u{0008} \t \n \u{000c} \r \" \' \\".into()
    ))]);
    toks!(r#""\nn""#, [Literal(Lit::Str("\nn".into()))]);
    reals!(r#""a" "b""#, [
        Literal(Lit::Str("a".into())), Literal(Lit::Str("b".into()))
    ]);

    toks!(r"'a'", [Literal(Lit::Char('a'))]);
    reals!(r#"'\b' '\t' '\n' '\f' '\r' '\"' '\'' '\\'"#, [
        Literal(Lit::Char('\u{0008}')),
        Literal(Lit::Char('\t')),
        Literal(Lit::Char('\n')),
        Literal(Lit::Char('\u{000c}')),
        Literal(Lit::Char('\r')),
        Literal(Lit::Char('"')),
        Literal(Lit::Char('\'')),
        Literal(Lit::Char('\\'))
    ]);
    reals!(r"'a' 'b'", [Literal(Lit::Char('a')), Literal(Lit::Char('b'))]);
}

#[test]
fn unicode_escapes() {
    // correct
    assert_eq!(spans(r"z\u0078z"), vec![
        TokenSpan {
            tok: Ident("zxz".into()),
            span: Span { lo: 0, hi: 7 }
        }
    ]);
    // too few hex digits
    assert_eq!(spans(r"z\u00z"), vec![
        TokenSpan {
            tok: Ident("zz".into()),
            span: Span { lo: 0, hi: 5 }
        }
    ]);
    // value is not a valid unicode scalar
    assert_eq!(spans(r"z\udecez"), vec![
        TokenSpan {
            tok: Ident("zz".into()),
            span: Span { lo: 0, hi: 7 }
        }
    ]);
    // correct with multiple 'u's
    assert_eq!(spans(r"z\uuuu0078z"), vec![
        TokenSpan {
            tok: Ident("zxz".into()),
            span: Span { lo: 0, hi: 10 }
        }
    ]);
    // backslashes that are not eligible
    // currently the lexer stops at backslash... enable this test again later!
    // assert_eq!(spans(r"z\\uuuu0078z"), vec![
    //     TokenSpan {
    //         tok: Ident(r"z\\uuuu0078z".into()),
    //         span: Span { lo: 0, hi: 11 }
    //     }
    // ]);
}

#[test]
fn new_lines() {
    assert_eq!(spans("abc \n xyz"), vec![
        TokenSpan {
            tok: Ident("abc".into()),
            span: Span { lo: 0, hi: 2 }
        },
        TokenSpan {
            tok: Whitespace,
            span: Span { lo: 3, hi: 5 }
        },
        TokenSpan {
            tok: Ident("xyz".into()),
            span: Span { lo: 6, hi: 8 }
        }
    ]);

    assert_eq!(spans("abc \r\n xyz"), vec![
        TokenSpan {
            tok: Ident("abc".into()),
            span: Span { lo: 0, hi: 2 }
        },
        TokenSpan {
            tok: Whitespace,
            span: Span { lo: 3, hi: 6 }
        },
        TokenSpan {
            tok: Ident("xyz".into()),
            span: Span { lo: 7, hi: 9 }
        }
    ]);
}

#[test]
fn basic_spans() {
    assert_eq!(spans("abc xyz"), vec![
        TokenSpan {
            tok: Ident("abc".into()),
            span: Span { lo: 0, hi: 2 }
        },
        TokenSpan {
            tok: Whitespace,
            span: Span { lo: 3, hi: 3 }
        },
        TokenSpan {
            tok: Ident("xyz".into()),
            span: Span { lo: 4, hi: 6 }
        }
    ]);

    assert_eq!(spans(".xxxx=="), vec![
        TokenSpan {
            tok: Dot,
            span: Span { lo: 0, hi: 0 }
        },
        TokenSpan {
            tok: Ident("xxxx".into()),
            span: Span { lo: 1, hi: 4 }
        },
        TokenSpan {
            tok: EqEq,
            span: Span { lo: 5, hi: 6 }
        }
    ]);

    assert_eq!(spans("     !"), vec![
        TokenSpan {
            tok: Whitespace,
            span: Span { lo: 0, hi: 4 }
        },
        TokenSpan {
            tok: Bang,
            span: Span { lo: 5, hi: 5 }
        }
    ]);
}
