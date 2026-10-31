#![allow(unused_must_use)]

use crate::util::dump;
use expect_test::expect;

#[test]
fn test_loop() {
  let mut out = String::new();

  dump(&mut out, "
    fun foo() { loop { break 1 } }
    fun bar() { loop { continue } }
    fun baz() { loop { return 1 } }
    fun qux() { let x, y = loop { break 1, 2 } return x + y }
  ");

  expect![[r#"
      === fun foo ===
      %0 LABEL 0
      %1 ==> GOTO %2
      %2 LABEL 0
      %3 = 1
      %4 PUT 0 %3
      %5 RET
      === fun bar ===
      %6 LABEL 0
      %7 ==> GOTO %8
      %8 LABEL 0
      %9 ==> GOTO %8
      === fun baz ===
      %10 LABEL 0
      %11 ==> GOTO %12
      %12 LABEL 0
      %13 = 1
      %14 PUT 0 %13
      %15 RET
      === fun qux ===
      %16 LABEL 0
      %17 ==> GOTO %18
      %18 LABEL 0
      %19 = 1
      %20 = 2
      %21 PUT 0 %19
      %22 PUT 1 %20
      %23 ==> GOTO %24
      %24 LABEL 2
      %25 = POP 0
      %26 = POP 1
      %27 = %25 + %26
      %28 PUT 0 %27
      %29 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
