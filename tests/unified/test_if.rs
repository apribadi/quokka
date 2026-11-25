#![allow(unused_must_use)]

use crate::util::test;
use expect_test::expect;

#[test]
fn test_if() {
  let mut out = String::new();

  test(&mut out, "
    fun foo(x, y, z) {
      let a =
        if (x) {
          1
        } else { if (y) {
          2
        } else { if (z) {
          3
        } else {
          4
        } } }
      return a + 5
    }
  ");

  expect![[r#"
      === fun foo ===
      %0 LABEL 3 0
      %1 = POP 0
      %2 = POP 1
      %3 = POP 2
      %4 COND %1 ==> %5 %21
      %5 LABEL 0 0
      %6 COND %2 ==> %7 %17
      %7 LABEL 0 0
      %8 COND %3 ==> %9 %13
      %9 LABEL 0 1
      %10 = 4
      %11 PUT 0 %10
      %12 JUMP ==> %25
      %13 LABEL 0 1
      %14 = 3
      %15 PUT 0 %14
      %16 JUMP ==> %25
      %17 LABEL 0 1
      %18 = 2
      %19 PUT 0 %18
      %20 JUMP ==> %25
      %21 LABEL 0 1
      %22 = 1
      %23 PUT 0 %22
      %24 JUMP ==> %25
      %25 LABEL 1 1
      %26 = POP 0
      %27 = 5
      %28 = %26 + %27
      %29 PUT 0 %28
      %30 RET
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_short_circuit() {
  let mut out = String::new();

  test(&mut out, "
    fun foo(x, y, z) {
      if (x && y && z) {
        1
      } else {
        2
      }
    }
  ");

  expect![[r#"
      === fun foo ===
      %0 LABEL 3 0
      %1 = POP 0
      %2 = POP 1
      %3 = POP 2
      %4 COND %1 ==> %5 %9
      %5 LABEL 0 1
      %6 = false
      %7 PUT 0 %6
      %8 JUMP ==> %18
      %9 LABEL 0 0
      %10 COND %2 ==> %11 %15
      %11 LABEL 0 1
      %12 = false
      %13 PUT 0 %12
      %14 JUMP ==> %18
      %15 LABEL 0 1
      %16 PUT 0 %3
      %17 JUMP ==> %18
      %18 LABEL 1 0
      %19 = POP 0
      %20 COND %19 ==> %21 %25
      %21 LABEL 0 1
      %22 = 2
      %23 PUT 0 %22
      %24 RET
      %25 LABEL 0 1
      %26 = 1
      %27 PUT 0 %26
      %28 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
