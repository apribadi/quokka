#![allow(unused_must_use)]

use crate::util::dump;
use expect_test::expect;

#[test]
fn test_dead_code() {
  let mut out = String::new();

  dump(&mut out, "
    fun foo(p) {
      if p {
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
      %2 COND %1
      %3 ==> GOTO %5
      %4 ==> GOTO %9
      %5 LABEL 0 1
      %6 = 2
      %7 PUT 0 %6
      %8 RET
      %9 LABEL 0 1
      %10 = 1
      %11 PUT 0 %10
      %12 RET
      %13 LABEL 0 1
      %14 = 3
      %15 PUT 0 %14
      %16 RET
      === fun id ===
      %17 LABEL 1 1
      %18 = POP 0
      %19 PUT 0 %18
      %20 RET

      SIMPLIFIED

      === fun foo ===
      %0 LABEL 1 0
      %1 = POP 0
      %2 COND %1
      %3 ==> GOTO %5
      %4 ==> GOTO %9
      %5 LABEL 0 1
      %6 = 2
      %7 PUT 0 %6
      %8 RET
      %9 LABEL 0 1
      %10 = 1
      %11 PUT 0 %10
      %12 RET
      === fun id ===
      %13 LABEL 1 1
      %14 = POP 0
      %15 PUT 0 %14
      %16 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
