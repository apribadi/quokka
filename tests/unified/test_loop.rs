#![allow(unused_must_use)]

use crate::util::test;
use expect_test::expect;

#[test]
fn test_loop() {
  let mut out = String::new();

  test(
    &mut out,
    "
    fun foo() { loop { break 1 } }
    fun bar() { loop { continue } }
    fun baz() { loop { return 1 } }
    fun qux() { let x, y = loop { break 1, 2 } return x + y }
  ",
  );

  expect![[r#"
      === fun foo ===
      %0 LABEL 0 0
      %1 = 1
      %2 PUT 0 %1
      %3 RET
      === fun bar ===
      %4 LABEL 0 0
      %5 JUMP ==> %6
      %6 LABEL 0 0
      %7 JUMP ==> %6
      === fun baz ===
      %8 LABEL 0 0
      %9 = 1
      %10 PUT 0 %9
      %11 RET
      === fun qux ===
      %12 LABEL 0 0
      %13 = 1
      %14 = 2
      %15 = OP2 %13 + %14
      %16 PUT 0 %15
      %17 RET
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_loop_nest() {
  let mut out = String::new();

  test(
    &mut out,
    "
    fun foo(x) {
      let y =
        loop {
          let y =
            loop {
              let y = loop { break x }
              break y
            }
          break y
        }
      return y
    }
  ",
  );

  expect![[r#"
      === fun foo ===
      %0 LABEL 1 0
      %1 = POP 0
      %2 PUT 0 %1
      %3 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
