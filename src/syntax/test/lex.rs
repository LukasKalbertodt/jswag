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
        Literal(Lit::Integer("1".into(), false, 10)),
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
        Literal(Lit::Integer("123".into(), false, 10))
    ]);
    assert_eq!(toks("123l"), vec![
        Literal(Lit::Integer("123".into(), true, 10))
    ]);
    assert_eq!(toks("0123"), vec![
        Literal(Lit::Integer("123".into(), false, 8))
    ]);
    assert_eq!(toks("0x1fa3l"), vec![
        Literal(Lit::Integer("1fa3".into(), true, 16))
    ]);
    assert_eq!(toks("0x1f"), vec![
        Literal(Lit::Integer("1f".into(), false, 16))
    ]);
    assert_eq!(toks("0b101l"), vec![
        Literal(Lit::Integer("101".into(), true, 2))
    ]);
    assert_eq!(toks("0l"), vec![
        Literal(Lit::Integer("0".into(), true, 10))
    ]);
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
