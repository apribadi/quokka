#![allow(unused_must_use)]

use crate::util::dump;
use expect_test::expect;

#[test]
fn test_fib_loop() {
  let mut out = String::new();

  dump(&mut out, "
    fun fib(n) {
      let mut a = 1
      let mut b = 0
      let mut n = n
      loop {
        if n == 0 { return b }
        let c = a + b
        a = b
        b = c
        n = n - 1
      }
    }
  ");

  expect![[r#"
      %0 LABEL 1
      %1 = ARG 0
      %2 = 1
      %3 = DEF-LOCAL %2
      %4 = 0
      %5 = DEF-LOCAL %4
      %6 = DEF-LOCAL %1
      %7 ==> GOTO %8
      %8 LABEL 0
      %9 = [ %6 ]
      %10 = 0
      %11 = %9 == %10
      %12 COND %11
      %13 ==> GOTO %19
      %14 ==> GOTO %15
      %15 LABEL 0
      %16 = [ %5 ]
      %17 PUT 0 %16
      %18 RET
      %19 LABEL 0
      %20 = [ %3 ]
      %21 = [ %5 ]
      %22 = %20 + %21
      %23 = [ %5 ]
      %24 [ %3 ] <- %23
      %25 [ %5 ] <- %22
      %26 = [ %6 ]
      %27 = 1
      %28 = %26 - %27
      %29 [ %6 ] <- %28
      %30 ==> GOTO %8
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_fib_while() {
  let mut out = String::new();

  dump(&mut out, "
    fun fib(n) {
      let mut a = 1
      let mut b = 0
      let mut n = n
      while n != 0 {
        let c = a + b
        a = b
        b = c
        n = n - 1
      }
      b
    }
  ");

  expect![[r#"
      %0 LABEL 1
      %1 = ARG 0
      %2 = 1
      %3 = DEF-LOCAL %2
      %4 = 0
      %5 = DEF-LOCAL %4
      %6 = DEF-LOCAL %1
      %7 ==> GOTO %8
      %8 LABEL 0
      %9 = [ %6 ]
      %10 = 0
      %11 = %9 == %10
      %12 = ! %11
      %13 COND %12
      %14 ==> GOTO %28
      %15 ==> GOTO %16
      %16 LABEL 0
      %17 = [ %3 ]
      %18 = [ %5 ]
      %19 = %17 + %18
      %20 = [ %5 ]
      %21 [ %3 ] <- %20
      %22 [ %5 ] <- %19
      %23 = [ %6 ]
      %24 = 1
      %25 = %23 - %24
      %26 [ %6 ] <- %25
      %27 ==> GOTO %8
      %28 LABEL 0
      %29 = [ %5 ]
      %30 PUT 0 %29
      %31 RET
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_fib_tailcall() {
  let mut out = String::new();

  dump(&mut out, "
    fun aux(a, b, n) {
      if n == 0 {
        b
      } else {
        aux(b, a + b, n - 1)
      }
    }

    fun fib(n) {
      aux(1, 0, n)
    }
  ");

  expect![[r#"
      %0 LABEL 3
      %1 = ARG 0
      %2 = ARG 1
      %3 = ARG 2
      %4 = 0
      %5 = %3 == %4
      %6 COND %5
      %7 ==> GOTO %9
      %8 ==> GOTO %18
      %9 LABEL 0
      %10 = %1 + %2
      %11 = 1
      %12 = %3 - %11
      %13 = CONST aux
      %14 PUT 0 %2
      %15 PUT 1 %10
      %16 PUT 2 %12
      %17 TAIL-CALL %13
      %18 LABEL 0
      %19 PUT 0 %2
      %20 RET
      %21 LABEL 1
      %22 = ARG 0
      %23 = 1
      %24 = 0
      %25 = CONST aux
      %26 PUT 0 %23
      %27 PUT 1 %24
      %28 PUT 2 %22
      %29 TAIL-CALL %25
  "#]].assert_eq(out.drain(..).as_ref());
}
