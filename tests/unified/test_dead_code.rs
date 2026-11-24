#![allow(unused_must_use)]

use crate::util::test;
use expect_test::expect;

#[test]
fn test_dead_code() {
  let mut out = String::new();

  test(&mut out, "
    fun foo(p) {
      if (p) {
        return 1
      } else {
        return 2
      }
      3
    }
    fun id(x) { x }
  ");

  expect![[r#"
      === fun foo ===
      %0 LABEL 1 0
      %1 = POP 0
      %2 COND %1 ==> %3 %7
      %3 LABEL 0 1
      %4 = 2
      %5 PUT 0 %4
      %6 RET
      %7 LABEL 0 1
      %8 = 1
      %9 PUT 0 %8
      %10 RET
      %11 LABEL 0 1
      %12 = 3
      %13 PUT 0 %12
      %14 RET
      === fun id ===
      %15 LABEL 1 1
      %16 = POP 0
      %17 PUT 0 %16
      %18 RET

      SIMPLIFIED

      === fun foo ===
      %0 LABEL 1 0
      %1 = POP 0
      %2 COND %1 ==> %3 %7
      %3 LABEL 0 1
      %4 = 2
      %5 PUT 0 %4
      %6 RET
      %7 LABEL 0 1
      %8 = 1
      %9 PUT 0 %8
      %10 RET
      === fun id ===
      %11 LABEL 1 1
      %12 = POP 0
      %13 PUT 0 %12
      %14 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
