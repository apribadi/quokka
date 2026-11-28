use quokka::lexer::scan;

#[test]
fn test_scan() {
  #![allow(unused_must_use)]

  use std::fmt::Write;
  let mut o = String::new();
  for (t, a, s) in scan(
    br#"
      app1(f)
      * /
      +++++ foo 100 +100 -=
      only-kebob-case
      _``'
      asdf@@@
      @@123 ; blah blah
      000
      0.0.0.0
      +0.0.0.0
      1.1.1.1
      -1.1.1.1
      a,b,c
      foo.bar.baz
      "foo\"".len
      ;"#,
  ) {
    write!(&mut o, "{:?} {:?} {:?}\n", str::from_utf8(s).unwrap(), t, a);
  }
  expect_test::expect![[r#"
      "app1" Symbol false
      "(" LParen true
      "f" Symbol true
      ")" RParen true
      "*" Mul false
      "/" Div false
      "+++++" Error false
      "foo" Symbol false
      "100" Int false
      "+100" Int false
      "-=" SubEqual false
      "only-kebob-case" Symbol false
      "_" Underscore false
      "`" Backtick true
      "`" Backtick true
      "'" Apostrophe true
      "asdf@@@" Error false
      "@@123" Error false
      "000" Error false
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
  }",
  ) {
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
      "-" Sub false
      "1" Int false
      ")" RParen true
      "+" Add false
      "fib" Symbol false
      "(" LParen true
      "n" Symbol true
      "-" Sub false
      "2" Int false
      ")" RParen true
      "}" RBrace false
  "#]].assert_eq(o.drain(..).as_str());
}
