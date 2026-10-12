//! lexical analysis of a byte-string to produce a stream of tokens

use crate::prelude::*;
use crate::token::Token;

const A: u8 = 10;
const B: u8 = 11;
const C: u8 = 12;
const D: u8 = 13;
const E: u8 = 14;
const F: u8 = 15;
const G: u8 = 16;
const H: u8 = 17;
const I: u8 = 18;

// ===== KINDS =====
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

static KIND: [u8; 256] = [
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

// ===== STATES =====
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

static TRAN: [[u8; 21]; 15] = [
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

/*
static FOO: [[u8; 16]; 256] = {
  let mut table = [[0u8; 16]; 256];
  let mut i = 0;
  while i < 256 {
    let mut j = 0;
    while j < 16 {
      table[i][j] = transition(KIND[i], j as u8);
      j += 1;
    }
    i += 1;
  }
  table
};
*/

/*
static TRAN: [[u8; 16]; 16] = [
// 0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // illegal
  [0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // space
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, E, 0, E, 0], // line feed
  [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, E, 3, E, 3], // semicolon
  [4, 4, 4, 3, 4, 4, 4, 4, 4, 4, 4, 4, E, 4, E, 4], // punctuation
  [5, 5, 2, 3, 5, 2, 2, 2, 2, 5, 5, D, E, D, E, 5], // sign
  [6, 6, 2, 3, 6, 2, 2, 2, 2, 6, 6, 6, E, 6, E, 6], // operator
  [7, 7, 2, 3, 7, 2, 2, 2, 2, 7, 7, D, E, D, E, 7], // dot
  [8, 8, 2, 3, 8, 2, 2, 2, 2, 8, 8, 8, E, 8, E, 8], // colon
  [A, A, A, 3, A, A, A, 9, 9, 9, 9, D, E, D, E, A], // backslash
  [A, A, A, 3, A, A, A, 9, 9, 9, 9, D, E, D, E, A], // letter
  [B, B, B, 3, B, D, B, 9, B, 9, 9, D, E, D, E, B], // digit
  [C, C, C, 3, C, C, C, C, C, C, C, C, F, C, F, C], // double quote
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // single quote
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // back quote
  [1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, E, 1, E, 1], // cont byte
];
*/

/*
pub struct Lexer<'a> {
  source: &'a [u8],
  index: usize,
  start: usize,
  stop: usize,
  state: u8,
  is_attached: bool,
  token: Token,
}

// 0 - skip
// 1 - token start
// 2 - token continue
static STATE_KIND: [u8; 16] = [
//0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
  0, 1, 2, 0, 1, 1, 1, 1, 1, 2, 1, 1, 1, 2, 2, 2
];

static TRANSITION: [[u8; 16]; 256] = {
  let mut t = [[0u8; 16]; 256];
  let mut i = 0;
  while i <= 255 {
    t[i] = TRANSITION_BY_BYTE_KIND[BYTE_KIND[i] as usize];
    i += 1;
  }
  t
};

fn is_start(x: u8) -> bool {
  STATE_KIND[(x & 0b1111) as usize] == 1
}

fn is_continue(x: u8) -> bool {
  STATE_KIND[(x & 0b1111) as usize] == 2
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a [u8]) -> Self {
    let mut t = Self {
      source,
      index: 0,
      start: 0,
      stop: 0,
      state: 0,
      is_attached: false,
      token: Token::Error,
    };
    t.next();
    t
  }

  pub fn token_start(&self) -> usize {
    self.start
  }

  pub fn token_stop(&self) -> usize {
    self.stop
  }

  pub fn token_is_attached(&self) -> bool {
    self.is_attached
  }

  pub fn token(&self) -> Token {
    self.token
  }

  pub fn token_span(&self) -> &'a [u8] {
    unsafe { self.source.get_unchecked(self.start .. self.stop) }
  }

  pub fn next(&mut self) {
    let n = self.source.len();
    let mut s = self.state;
    let mut i = self.index;

    let start =
      loop {
        if is_start(s) { break i - 1; }
        if i == n { s = 0; break i; }
        s = TRANSITION[unsafe { *self.source.get_unchecked(i) } as usize][s as usize];
        i += 1;
      };

    let mut last_state;
    let stop =
      loop {
        last_state = s;
        if i == n { s = 0; break i; }
        s = TRANSITION[unsafe { *self.source.get_unchecked(i) } as usize][s as usize];
        i += 1;
        if is_start(s) || ! is_continue(s) { break i - 1; }
      };

    let is_attached = start == self.stop;

    self.index = i;
    self.start = start;
    self.stop = stop;
    self.state = s;
    self.is_attached = is_attached;

    self.token =
      match last_state {
        0 => Token::Eof,
        F =>
          // can look at first char to see quote kind
          Token::DoubleQuote,
        4 | 5 | 6 | 7 | 8 =>
          unsafe { transmute::<u8, Token>(*self.source.get_unchecked(start)) },
        B | D =>
          Token::Number,
        A =>
          match unsafe { self.source.get_unchecked(start .. stop) } {
            b"_" => Token::Underscore,
            _ => Token::Symbol,
          },
        2 =>
          match unsafe { self.source.get_unchecked(start .. stop) } {
            b"&&" => Token::And,
            b"==" => Token::CmpEq,
            b">=" => Token::CmpGe,
            b"<=" => Token::CmpLe,
            b"!=" => Token::CmpNe,
            b"--" => Token::Dec,
            b"..." => Token::DotDotDot,
            b"++" => Token::Inc,
            b"||" => Token::Or,
            b"<<" => Token::Shl,
            b">>" => Token::Shr,
            _ => Token::Error,
          },
        9 =>
          match unsafe { self.source.get_unchecked(start .. stop) } {
            [b'.', ..] => Token::Field,
            [b':', ..] => Token::StaticField,
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
        _ => Token::Error,
      }
  }
}
*/
