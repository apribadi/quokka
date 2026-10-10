use crate::prelude::*;
use crate::ast;
use crate::token::Token;
use crate::token::scan;
use crate::syntax;

pub fn parse_stmt(source: &str) -> Result<ast::Stmt<'_>, ()> {
  let mut t = scan(source);
  let mut p = syntax::STMTParser::new();
  p.parse(source, t).map_err(|_| ())
}

#[test]
fn test_parse_expr() {
  let mut o = String::new();
  let ast =
    parse_stmt(
      "- loop {} + 2"
      );
  write!(&mut o, "{:?}\n", ast);
  expect_test::expect![[r#"
      Ok(ExprList([Op2((Add, Op1((Neg, Loop([]))), ConstI64(2)))]))
  "#]].assert_eq(o.drain(..).as_str());
}
