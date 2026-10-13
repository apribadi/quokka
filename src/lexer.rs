//! lexical analysis of a byte-string to produce a stream of tokens

use crate::prelude::*;
use crate::token::Token;
use std::mem::transmute;

const NUM_STATES: usize = 24;
const NUM_BYTE_KINDS: usize = 15;

const A: u8 = 10;
const B: u8 = 11;
const C: u8 = 12;
const D: u8 = 13;
const E: u8 = 14;
const F: u8 = 15;
const G: u8 = 16;
const H: u8 = 17;
const I: u8 = 18;
const J: u8 = 19;
const K: u8 = 20;
const L: u8 = 21;
const M: u8 = 22;
const N: u8 = 23;

// ===== BYTE KIND =====
//
// 0 - illegal
// 1 - space        HT SP
// 2 - newline      LF
// 3 - semicolon    ;
// 4 - punctuation  ' ( ) , [ ] ` { }
// 5 - dot          .
// 6 - colon        :
// 7 - plus         +
// 8 - hyphen       -
// 9 - operator     ! # $ % & * / < = > ? @ ^ _ | ~
// A - letter       A ... Z a ... z
// B - zero         0
// C - digit        1 2 3 4 5 6 7 8 9
// D - quote        "
// E - backslash    \

static BYTE_KIND: [u8; 256] = [
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  1, 9, D, 9, 9, 9, 9, 4, 4, 4, 9, 7, 4, 8, 5, 9,
  B, C, C, C, C, C, C, C, C, C, 6, 3, 9, 9, 9, 9,
  9, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, E, 4, 9, 9,
  4, A, A, A, A, A, A, A, A, A, A, A, A, A, A, A,
  A, A, A, A, A, A, A, A, A, A, A, 4, 9, 4, 9, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// ===== STATE =====
//
// 0 - nil
// 1 - illegal char start
// 2 - illegal char continue
// 3 - comment
// 4 - punctuation
// 5 - dot start
// 6 - colon start
// 7 - signed start
// 8 - signed zero
// 9 - operator start
// A - symbol start
// B - zero
// C - int start
// D - quote start
// E - quote escape
// F - quote continue
// G - quote stop
// H - dot continue
// I - colon continue
// J - operator continue
// K - int continue
// L - symbol continue
// M - reserved symbol
// N - reserved number

static TRANSITION_: [[u8; NUM_STATES]; NUM_BYTE_KINDS] = [
// 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F, G, H, I, J, K, L, M, N
  [1, 2, 2, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, F, F, F, 1, 1, 1, 1, 1, 1, 1, 1], // 0 - illegal
  [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, F, F, F, 0, 0, 0, 0, 0, 0, 0, 0], // 1 - space
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, F, F, F, 0, 0, 0, 0, 0, 0, 0, 0], // 2 - newline
  [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, F, F, F, 3, 3, 3, 3, 3, 3, 3, 3], // 3 - semicolon
  [4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, F, F, F, 4, 4, 4, 4, 4, 4, 4, 4], // 4 - punctuation
  [5, 5, 5, 3, 5, H, 5, 5, N, 5, 5, N, N, F, F, F, 5, H, 5, 5, N, 5, 5, N], // 5 - dot
  [6, 6, 6, 3, 6, 6, I, 6, 6, 6, 6, 6, 6, F, F, F, 6, 6, I, 6, 6, 6, 6, 6], // 6 - colon
  [7, 7, 7, 3, 7, 7, 7, J, N, J, M, N, N, F, F, F, 7, 7, 7, J, N, M, M, N], // 7 - plus
  [7, 7, 7, 3, 7, 7, 7, J, N, J, L, N, N, F, F, F, 7, 7, 7, J, N, L, M, N], // 8 - hyphen
  [9, 9, 9, 3, 9, 9, 9, J, N, J, M, N, N, F, F, F, 9, 9, 9, J, N, M, M, N], // 9 - operator
  [A, A, A, 3, A, A, A, M, N, M, L, N, N, F, F, F, A, A, A, M, N, L, M, N], // A - letter
  [B, B, B, 3, B, B, B, 8, N, M, M, N, K, F, F, F, B, B, B, M, K, M, M, N], // B - zero
  [C, C, C, 3, C, C, C, K, N, M, M, N, K, F, F, F, C, C, C, M, K, M, M, N], // C - digit
  [D, D, D, 3, D, D, D, D, D, D, D, D, D, G, F, G, D, D, D, D, D, D, D, D], // D - quote
  [1, 2, 2, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, E, F, E, 1, 1, 1, 1, 1, 1, 1, 1], // E - backslash
];

// ===== STATE KIND =====
//
// 0 - skip
// 1 - token start
// 2 - token continue

static STATE_KIND: [u8; NUM_STATES] = [
//0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D, E, F, G, H, I, J, K, L, M, N
  0, 0, 0, 0, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2
];

static TRANSITION: [[u8; 32]; 256] = {
  let mut t = [[0u8; 32]; 256];
  let mut i = 0;
  while i < 256 {
    let mut j = 0;
    while j < NUM_STATES {
      t[i][j] = TRANSITION_[BYTE_KIND[i] as usize][j];
      j += 1;
    }
    i += 1;
  }
  t
};

pub fn scan(source: &[u8]) -> Box<[(Token, bool, &'_ [u8])]> {
  let t = &TRANSITION;
  let u = &STATE_KIND;
  let n = source.len();
  let mut a = Vec::new();
  let mut s = 0u8;
  let mut j = 0;
  let mut k = 0;
  let mut p = false;
  for i in 0 .. n {
    let x = unsafe { *t[*source.get_unchecked(i) as usize].get_unchecked(s as usize) };
    let y = unsafe { *u.get_unchecked(x as usize) };
    if p && y != 2 {
      p = false;
      let z = unsafe { source.get_unchecked(k .. i) };
      a.push((classify(s, z), k == j, z));
      j = i;
    }
    if y == 1 {
      p = true;
      k = i;
    }
    s = x;
  }
  if p {
    let z = unsafe { source.get_unchecked(k .. n) };
    a.push((classify(s, z), k == j, z));
  }
  a.into_boxed_slice()
}

fn classify(last_state: u8, span: &[u8]) -> Token {
  match last_state {
    4 | 5 | 6 | 7 =>
      unsafe { transmute::<u8, Token>(*span.get_unchecked(0)) },
    8 | B | C | K =>
      Token::Int,
    H =>
      match span.len() {
        3 => Token::DotDotDot,
        _ => Token::Error,
      }
    9 | J =>
      match span {
        b"_" => Token::Underscore,
        b"==" => Token::CmpEq,
        b">=" => Token::CmpGe,
        b"<=" => Token::CmpLe,
        b"!=" => Token::CmpNe,
        b"--" => Token::Dec,
        b"++" => Token::Inc,
        b"&&" => Token::LogAnd,
        b"||" => Token::LogOr,
        b"<<" => Token::Shl,
        b">>" => Token::Shr,
        _ => Token::Error,
      },
    A | L =>
      match span {
        b"break" => Token::Break,
        b"continue" => Token::Continue,
        b"do" => Token::Do,
        b"elif" => Token::Elif,
        b"else" => Token::Else,
        b"false" => Token::False,
        b"for" => Token::For,
        b"fun" => Token::Fun,
        b"if" => Token::If,
        b"let" => Token::Let,
        b"loop" => Token::Loop,
        b"return" => Token::Return,
        b"true" => Token::True,
        b"var" => Token::Var,
        b"while" => Token::While,
        _ => Token::Symbol,
      },
    G =>
      Token::Quote,
    _ =>
      Token::Error,
  }
}

#[test]
fn test_scan() {
  use std::fmt::Write;
  let mut o = String::new();
  for (t, a, s) in scan(
    br#"+++++ foo 100 +100 -=
      only-kebob-case
      asdf@@@
      @@123 ; blah blah
      0.0.0.0
      +0.0.0.0
      1.1.1.1
      -1.1.1.1
      a,b,c
      foo.bar.baz
      "foo\"".len
      ;"#
    )
  {
    write!(&mut o, "{:?} {:?} {:?}\n", str::from_utf8(s).unwrap(), t, a);
  }
  expect_test::expect![[r#"
      "+++++" Error true
      "foo" Symbol false
      "100" Int false
      "+100" Int false
      "-=" Error false
      "only-kebob-case" Symbol false
      "asdf@@@" Error false
      "@@123" Error false
      "0.0.0.0" Error false
      "+0.0.0.0" Error false
      "1.1.1.1" Error false
      "-1.1.1.1" Error false
      "a" Symbol false
      "," Comma true
      "b" Symbol true
      "," Comma true
      "c" Symbol true
      "foo" Symbol false
      "." Dot true
      "bar" Symbol true
      "." Dot true
      "baz" Symbol true
      "\"foo\\\"\"" Quote false
      "." Dot true
      "len" Symbol true
  "#]].assert_eq(o.drain(..).as_str());
  for (t, a, s) in scan(
      b"
fun fib(n) {
  if n <= 2 {
    return n
  } else {
    return fib(n - 1) + fib(n - 2)
  }"
    )
  {
    write!(&mut o, "{:?} {:?} {:?}\n", str::from_utf8(s).unwrap(), t, a);
  }
  expect_test::expect![[r#"
      "fun" Fun false
      "fib" Symbol false
      "(" LParen true
      "n" Symbol true
      ")" RParen true
      "{" LBrace false
      "if" If false
      "n" Symbol false
      "<=" CmpLe false
      "2" Int false
      "{" LBrace false
      "return" Return false
      "n" Symbol false
      "}" RBrace false
      "else" Else false
      "{" LBrace false
      "return" Return false
      "fib" Symbol false
      "(" LParen true
      "n" Symbol true
      "-" Hyphen false
      "1" Int false
      ")" RParen true
      "+" Add false
      "fib" Symbol false
      "(" LParen true
      "n" Symbol true
      "-" Hyphen false
      "2" Int false
      ")" RParen true
      "}" RBrace false
  "#]].assert_eq(o.drain(..).as_str());
}
