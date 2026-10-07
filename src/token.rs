use logos::Logos;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[derive(Logos)]
#[logos(utf8 = false)]
pub enum Token {
  #[token(r"!")] Not,
  #[token(r"$")] Dollar,
  #[token(r"%")] Rem,
  #[token(r"&")] BitAnd,
  #[token(r"(")] LParen,
  #[token(r")")] RParen,
  #[token(r"*")] Mul,
  #[token(r"+")] Add,
  #[token(r",")] Comma,
  #[token(r"-")] Hyphen,
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

  #[regex(r"([[:space:]]+|;[^\n]*)+")]
  #[regex(r"[!$%&*+\-/<=>?@\^_|~[:alnum:]]+", |_| false, priority = 0)]
  #[regex(r"\.+", |_| false, priority = 0)]
  #[regex(r":+", |_| false, priority = 0)]
  Space,

  /*
  Number,
  DoubleQuote,
  */
}

impl std::fmt::Display for Token {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[test]
fn test_foo() {
  use std::fmt::Write;
  let mut o = String::new();
  let mut t =
    Token::lexer(
b"
: :: :::
-foo-!!
!-foo
---0
++ -- foo bar fun ; blah
:foo .bar
-=
---foo
. .. ... ....
foo-bar
foo+++bar +++
foo---bar ---
foo^^^bar ^^^
foo&&&bar &&&
foo|||bar |||

  qux
+; blah"
      );
  while let Some(tok) = t.next() {
    write!(&mut o, "{} {:?}\n", tok, str::from_utf8(t.slice()).unwrap());
  }
  expect_test::expect![[r#"
      Ok(Space) "\n"
      Ok(Colon) ":"
      Ok(Space) " "
      Ok(BadColons) "::"
      Ok(Space) " "
      Ok(BadColons) ":::"
      Ok(Space) "\n"
      Err(()) "-foo-!!"
      Ok(Space) "\n"
      Err(()) "!-foo"
      Ok(Space) "\n"
      Err(()) "---0"
      Ok(Space) "\n"
      Ok(Inc) "++"
      Ok(Space) " "
      Ok(Dec) "--"
      Ok(Space) " "
      Ok(Symbol) "foo"
      Ok(Space) " "
      Ok(Symbol) "bar"
      Ok(Space) " "
      Ok(Fun) "fun"
      Ok(Space) " ; blah\n"
      Ok(StaticField) ":foo"
      Ok(Space) " "
      Ok(Field) ".bar"
      Ok(Space) "\n"
      Err(()) "-="
      Ok(Space) "\n"
      Ok(Symbol) "---foo"
      Ok(Space) "\n"
      Ok(Dot) "."
      Ok(Space) " "
      Ok(BadDots) ".."
      Ok(Space) " "
      Ok(DotDotDot) "..."
      Ok(Space) " "
      Ok(BadDots) "...."
      Ok(Space) "\n"
      Ok(Symbol) "foo-bar"
      Ok(Space) "\n"
      Err(()) "foo+++bar"
      Ok(Space) " "
      Err(()) "+++"
      Ok(Space) "\n"
      Ok(Symbol) "foo---bar"
      Ok(Space) " "
      Err(()) "---"
      Ok(Space) "\n"
      Err(()) "foo^^^bar"
      Ok(Space) " "
      Err(()) "^^^"
      Ok(Space) "\n"
      Err(()) "foo&&&bar"
      Ok(Space) " "
      Err(()) "&&&"
      Ok(Space) "\n"
      Err(()) "foo|||bar"
      Ok(Space) " "
      Err(()) "|||"
      Ok(Space) "\n\n  "
      Ok(Symbol) "qux"
      Ok(Space) "\n"
      Ok(Add) "+"
      Ok(Space) "; blah\n"
  "#]].assert_eq(o.drain(..).as_str());
}
