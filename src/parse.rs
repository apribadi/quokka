use crate::prelude::*;
use crate::ast;
use crate::token::Token;
use crate::token::scan;
use crate::syntax;

pub fn parse_expr(source: &str) -> Result<ast::Expr, ()> {
  let mut t = scan(source);
  let mut p = syntax::EXPRParser::new();
  p.parse(t).map_err(|_| ())
}

#[test]
fn test_parse_expr() {
  let mut o = String::new();
  let ast =
    parse_expr(
"true * false + true(false) "
      );
  write!(&mut o, "{:?}\n", ast);
  expect_test::expect![[r#"
      Ok(Op2((Add, Op2((Mul, ConstBool(true), ConstBool(false))), Call((ConstBool(true), [ConstBool(false)])))))
  "#]].assert_eq(o.drain(..).as_str());
}
