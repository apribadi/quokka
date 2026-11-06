#![allow(unused_must_use)]

use crate::util::dump;
use expect_test::expect;

#[test]
fn test_projection() {
  let mut out = String::new();

  dump(&mut out, "
    fun foo(x) {
      x.a.b[1].c(2, 3)
    }
  ");

  expect![[r#"
      === fun foo ===
      %0 LABEL 1 2
      %1 = POP 0
      %2 = 2
      %3 = 3
      %4 = 1
      %5 = %1 [ .a ]
      %6 = %5 [ .b ]
      %7 = %6 [ %4 ]
      %8 = %7 [ .c ]
      %9 PUT 0 %2
      %10 PUT 1 %3
      %11 TAIL-CALL %8

      SIMPLIFIED

      === fun foo ===
      %0 LABEL 1 2
      %1 = POP 0
      %2 = 2
      %3 = 3
      %4 = 1
      %5 = %1 [ .a ]
      %6 = %5 [ .b ]
      %7 = %6 [ %4 ]
      %8 = %7 [ .c ]
      %9 PUT 0 %2
      %10 PUT 1 %3
      %11 TAIL-CALL %8
  "#]].assert_eq(out.drain(..).as_ref());
}
