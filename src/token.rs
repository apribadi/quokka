use logos::Logos;
use logos::Lexer;

pub fn scan(source: &str) -> impl Iterator<Item = Result<(usize, Token, usize), ()>> {
  Token::lexer(source).spanned().map(|(token, span)| 
    match token {
      Err(()) => Err(()),
      Ok(token) => Ok((span.start, token, span.end)),
    }
  )
}

fn is_attached<'a>(t: &Lexer<'a, Token>) -> bool {
  t.extras != t.span().start
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[derive(Logos)]
#[logos(extras = usize)]
#[logos(skip(r"([ \t\n]+|;[^\n]*)+", |t| t.extras = t.span().end))]
#[logos(subpattern symbolic = r"[\-\^#!$%&*+/<=>?@_|~[:alnum:]]")]
pub enum Token {
  #[token(r"!")] Not,
  #[token(r"$")] Dollar,
  #[token(r"%")] Rem,
  #[token(r"&")] BitAnd,
  #[token(r"(", is_attached)] LParen(bool),
  #[token(r")")] RParen,
  #[token(r"*")] Mul,
  #[token(r"+")] Add,
  #[token(r",")] Comma,
  #[token(r"-")] Sub,
  #[token(r".")] Dot,
  #[token(r"/")] Div,
  #[token(r":")] Colon,
  #[token(r"<")] CmpLt,
  #[token(r"=")] Equal,
  #[token(r">")] CmpGt,
  #[token(r"?")] Query,
  #[token(r"@")] At,
  #[token(r"[")] LBracket,
  #[token(r"]")] RBracket,
  #[token(r"^")] BitXor,
  #[token(r"_")] Underscore,
  #[token(r"{")] LBrace,
  #[token(r"|")] BitOr,
  #[token(r"}")] RBrace,
  #[token(r"~")] Tilde,
  #[token(r"<<")] Shl,
  #[token(r">>")] Shr,
  #[token(r"&&")] And,
  #[token(r"||")] Or,
  #[token(r"++")] Inc,
  #[token(r"--")] Dec,
  #[token(r"==")] CmpEq,
  #[token(r"!=")] CmpNe,
  #[token(r"<=")] CmpLe,
  #[token(r">=")] CmpGe,
  #[token(r"...")] DotDotDot,
  #[token(r"break")] Break,
  #[token(r"continue")] Continue,
  #[token(r"do")] Do,
  #[token(r"elif")] Elif,
  #[token(r"else")] Else,
  #[token(r"false")] False,
  #[token(r"for")] For,
  #[token(r"fun")] Fun,
  #[token(r"if")] If,
  #[token(r"let")] Let,
  #[token(r"loop")] Loop,
  #[token(r"mutable")] Mutable,
  #[token(r"return")] Return,
  #[token(r"true")] True,
  #[token(r"while")] While,
  #[regex(r"-*[[:alpha:]][\-[:alnum:]]*", priority = 1)] Symbol,
  #[regex(r"[\-+]?(0|[1-9][0-9]*)")] Int,
  #[regex(r"\.+|:+|(?&symbolic)+|[\-+]?[0-9](\.|:|(?&symbolic))*", |_| false, priority = 0)] Malformed,
  /*
  DoubleQuote,
  */
}

#[test]
fn test_app() {
  use std::fmt::Write;
  let mut o = String::new();
  let mut t = Token::lexer("true(false) true (false)");
  while let Some(tok) = t.next() {
    write!(&mut o, "{:?} {:?}\n", tok, t.slice());
  }
  expect_test::expect![[r#"
      Ok(True) "true"
      Ok(LParen(true)) "("
      Ok(False) "false"
      Ok(RParen) ")"
      Ok(True) "true"
      Ok(LParen(false)) "("
      Ok(False) "false"
      Ok(RParen) ")"
  "#]].assert_eq(o.drain(..).as_str());
}

#[test]
fn test_bad() {
  use std::fmt::Write;
  let mut o = String::new();
  let mut t =
    Token::lexer(
"
0:0:0
foo.0.0
bar.+0.0
0_0
0_0.abc
100
-100
+100
001
000
-001
foo+++bar
0.0.0
+0xcafe
-0
a.b
a.++b
0.++b
;"
      );
  while let Some(tok) = t.next() {
    write!(&mut o, "{:?} {:?}\n", tok, t.slice());
  }
  expect_test::expect![[r#"
      Err(()) "0:0:0"
      Ok(Symbol) "foo"
      Ok(Dot) "."
      Err(()) "0.0"
      Ok(Symbol) "bar"
      Ok(Dot) "."
      Err(()) "+0.0"
      Err(()) "0_0"
      Err(()) "0_0.abc"
      Ok(Int) "100"
      Ok(Int) "-100"
      Ok(Int) "+100"
      Err(()) "001"
      Err(()) "000"
      Err(()) "-001"
      Err(()) "foo+++bar"
      Err(()) "0.0.0"
      Err(()) "+0xcafe"
      Ok(Int) "-0"
      Ok(Symbol) "a"
      Ok(Dot) "."
      Ok(Symbol) "b"
      Ok(Symbol) "a"
      Ok(Dot) "."
      Err(()) "++b"
      Err(()) "0.++b"
  "#]].assert_eq(o.drain(..).as_str());
}

#[test]
fn test_fib() {
  use std::fmt::Write;
  let mut o = String::new();
  let mut t =
    Token::lexer(
"
fun fib(n) {
  if (n <= 1) { return 1 }
  return fib(n - 1) + fib(n - 2) ; recurse
}
"
      );
  while let Some(tok) = t.next() {
    write!(&mut o, "{:?} {:?}\n", tok, t.slice());
  }
  expect_test::expect![[r#"
      Ok(Fun) "fun"
      Ok(Symbol) "fib"
      Ok(LParen(true)) "("
      Ok(Symbol) "n"
      Ok(RParen) ")"
      Ok(LBrace) "{"
      Ok(If) "if"
      Ok(LParen(false)) "("
      Ok(Symbol) "n"
      Ok(CmpLe) "<="
      Ok(Int) "1"
      Ok(RParen) ")"
      Ok(LBrace) "{"
      Ok(Return) "return"
      Ok(Int) "1"
      Ok(RBrace) "}"
      Ok(Return) "return"
      Ok(Symbol) "fib"
      Ok(LParen(true)) "("
      Ok(Symbol) "n"
      Ok(Sub) "-"
      Ok(Int) "1"
      Ok(RParen) ")"
      Ok(Add) "+"
      Ok(Symbol) "fib"
      Ok(LParen(true)) "("
      Ok(Symbol) "n"
      Ok(Sub) "-"
      Ok(Int) "2"
      Ok(RParen) ")"
      Ok(RBrace) "}"
  "#]].assert_eq(o.drain(..).as_str());
}
