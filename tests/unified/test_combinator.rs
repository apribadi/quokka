#![allow(unused_must_use)]

use crate::util;
use expect_test::expect;

#[test]
fn test_select() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(x, y, z) {
      x ? y : z
    }
  ");

  expect![[r#"
      %0 LABEL 3
      %1 = ARG 0
      %2 = ARG 1
      %3 = ARG 2
      %4 COND %1
      %5 ==> GOTO %7
      %6 ==> GOTO %10
      %7 LABEL 0
      %8 PUT 0 %3
      %9 RET
      %10 LABEL 0
      %11 PUT 0 %2
      %12 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
