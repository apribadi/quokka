#![allow(unused_must_use)]

use crate::util::dump;
use expect_test::expect;

#[test]
fn test_id() {
  let mut out = String::new();

  dump(&mut out, "
    fun id(x) {
      x
    }
  ");

  expect![[r#"
      === fun id ===
      %0 LABEL 1
      %1 = POP 0
      %2 PUT 0 %1
      %3 RET
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_apply() {
  let mut out = String::new();

  dump(&mut out, "
    fun app1(f, x) { f(x) }
    fun app2(f, x, y) { f(x, y) }
    fun id(x) { x }
    fun flip(x, y) { y, x }
    fun foo(x) { app1(id, x) }
    fun bar(x, y) { app2(flip, x, y) }
    fun baz(x) { app2(app1, id, x) }
  ");

  expect![[r#"
      === fun app1 ===
      %0 LABEL 2
      %1 = POP 0
      %2 = POP 1
      %3 PUT 0 %2
      %4 TAIL-CALL %1
      === fun app2 ===
      %5 LABEL 3
      %6 = POP 0
      %7 = POP 1
      %8 = POP 2
      %9 PUT 0 %7
      %10 PUT 1 %8
      %11 TAIL-CALL %6
      === fun id ===
      %12 LABEL 1
      %13 = POP 0
      %14 PUT 0 %13
      %15 RET
      === fun flip ===
      %16 LABEL 2
      %17 = POP 0
      %18 = POP 1
      %19 PUT 0 %18
      %20 PUT 1 %17
      %21 RET
      === fun foo ===
      %22 LABEL 1
      %23 = POP 0
      %24 = CONST id
      %25 = CONST app1
      %26 PUT 0 %24
      %27 PUT 1 %23
      %28 TAIL-CALL %25
      === fun bar ===
      %29 LABEL 2
      %30 = POP 0
      %31 = POP 1
      %32 = CONST flip
      %33 = CONST app2
      %34 PUT 0 %32
      %35 PUT 1 %30
      %36 PUT 2 %31
      %37 TAIL-CALL %33
      === fun baz ===
      %38 LABEL 1
      %39 = POP 0
      %40 = CONST app1
      %41 = CONST id
      %42 = CONST app2
      %43 PUT 0 %40
      %44 PUT 1 %41
      %45 PUT 2 %39
      %46 TAIL-CALL %42
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_select() {
  let mut out = String::new();

  dump(&mut out, "
    fun select(x, y, z) {
      x ? y : z
    }
  ");

  expect![[r#"
      === fun select ===
      %0 LABEL 3
      %1 = POP 0
      %2 = POP 1
      %3 = POP 2
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
