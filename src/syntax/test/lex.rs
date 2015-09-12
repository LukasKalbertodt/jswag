/// This module contains unit tests for the tokenizer.
///

use syntax::*;
use filemap::{FileMap, Span};
use std::rc::Rc;
use diagnostics::ErrorHandler;


fn toks(src: &str) -> Vec<Token> {
    let fmap = Rc::new(FileMap::new("<unit-test>".into(), src.into()));
    let error_handler = ErrorHandler::new(fmap.clone());
    let toks = Tokenizer::new(&fmap, &error_handler);
    toks.map(|ts| ts.tok).collect()
}

fn spans(src: &str) -> Vec<TokenSpan> {
    let fmap = Rc::new(FileMap::new("<unit-test>".into(), src.into()));
    let error_handler = ErrorHandler::new(fmap.clone());
    let toks = Tokenizer::new(&fmap, &error_handler);
    toks.collect()
}

#[test]
fn empty() {
    assert_eq!(toks(""), vec![]);
}

#[test]
fn idents() {
    assert_eq!(toks("foo"), vec![Token::Word("foo".into())]);
    assert_eq!(toks("foo bar"), vec![
        Token::Word("foo".into()),
        Token::Whitespace,
        Token::Word("bar".into()),
    ]);
    assert_eq!(toks("1bla"), vec![
        Token::Literal(Lit::Integer("1".into(), false, 10)),
        Token::Word("bla".into())
    ]);
    assert_eq!(toks("b1la"), vec![Token::Word("b1la".into())]);

}

#[test]
fn int_literals() {
    assert_eq!(toks("123"), vec![
        Token::Literal(Lit::Integer("123".into(), false, 10))
    ]);
    assert_eq!(toks("123l"), vec![
        Token::Literal(Lit::Integer("123".into(), true, 10))
    ]);
    assert_eq!(toks("0123"), vec![
        Token::Literal(Lit::Integer("123".into(), false, 8))
    ]);
    assert_eq!(toks("0x1fa3l"), vec![
        Token::Literal(Lit::Integer("1fa3".into(), true, 16))
    ]);
    assert_eq!(toks("0x1f"), vec![
        Token::Literal(Lit::Integer("1f".into(), false, 16))
    ]);
    assert_eq!(toks("0b101l"), vec![
        Token::Literal(Lit::Integer("101".into(), true, 2))
    ]);
    assert_eq!(toks("0l"), vec![
        Token::Literal(Lit::Integer("0".into(), true, 10))
    ]);
}

#[test]
fn unicode_escapes() {
    assert_eq!(spans(r"a\u0078a"), vec![
        TokenSpan {
            tok: Token::Word("axa".into()),
            span: Span { lo: 0, hi: 7 }
        }
    ]);
    assert_eq!(spans(r"a\u00_a"), vec![
        TokenSpan {
            tok: Token::Word("a_a".into()),
            span: Span { lo: 0, hi: 6 }
        }
    ]);
    assert_eq!(spans(r"a\udecea"), vec![
        TokenSpan {
            tok: Token::Word("aa".into()),
            span: Span { lo: 0, hi: 7 }
        }
    ]);
}

#[test]
fn basic_spans() {
    assert_eq!(spans("abc xyz"), vec![
        TokenSpan {
            tok: Token::Word("abc".into()),
            span: Span { lo: 0, hi: 2 }
        },
        TokenSpan {
            tok: Token::Whitespace,
            span: Span { lo: 3, hi: 3 }
        },
        TokenSpan {
            tok: Token::Word("xyz".into()),
            span: Span { lo: 4, hi: 6 }
        }
    ]);

    assert_eq!(spans(".xxxx=="), vec![
        TokenSpan {
            tok: Token::Dot,
            span: Span { lo: 0, hi: 0 }
        },
        TokenSpan {
            tok: Token::Word("xxxx".into()),
            span: Span { lo: 1, hi: 4 }
        },
        TokenSpan {
            tok: Token::EqEq,
            span: Span { lo: 5, hi: 6 }
        }
    ]);

    assert_eq!(spans("     !"), vec![
        TokenSpan {
            tok: Token::Whitespace,
            span: Span { lo: 0, hi: 4 }
        },
        TokenSpan {
            tok: Token::Bang,
            span: Span { lo: 5, hi: 5 }
        }
    ]);
}
