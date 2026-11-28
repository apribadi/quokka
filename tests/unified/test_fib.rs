#![allow(unused_must_use)]

use crate::util::test;
use expect_test::expect;

#[test]
fn test_fib_loop() {
  let mut out = String::new();

  test(&mut out, "
    fun fib(n) {
      let mut a = 1
      let mut b = 0
      let mut n = n
      loop {
        if (n == 0) { return b }
        let c = a + b
        a = b
        b = c
        n = n - 1
      }
    }
  ");

  expect![[r#"
      === fun fib ===
      %0 LABEL 1 0
      %1 = POP 0
      %2 = 1
      %3 = DEF-LOCAL %2
      %4 = 0
      %5 = DEF-LOCAL %4
      %6 = DEF-LOCAL %1
      %7 JUMP ==> %8
      %8 LABEL 0 0
      %9 = [ %6 ]
      %10 = 0
      %11 = CMP %9 == %10
      %12 COND %11 ==> %17 %13
      %13 LABEL 0 1
      %14 = [ %5 ]
      %15 PUT 0 %14
      %16 RET
      %17 LABEL 0 0
      %18 = [ %3 ]
      %19 = [ %5 ]
      %20 = OP2 %18 + %19
      %21 = [ %5 ]
      %22 [ %3 ] <- %21
      %23 [ %5 ] <- %20
      %24 = [ %6 ]
      %25 = 1
      %26 = OP2 %24 - %25
      %27 [ %6 ] <- %26
      %28 JUMP ==> %8
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_fib_while() {
  let mut out = String::new();

  test(&mut out, "
    fun fib(n) {
      let mut a = 1
      let mut b = 0
      let mut n = n
      while (n != 0) {
        let c = a + b
        a = b
        b = c
        n = n - 1
      }
      b
    }
  ");

  expect![[r#"
      === fun fib ===
      %0 LABEL 1 0
      %1 = POP 0
      %2 = 1
      %3 = DEF-LOCAL %2
      %4 = 0
      %5 = DEF-LOCAL %4
      %6 = DEF-LOCAL %1
      %7 JUMP ==> %8
      %8 LABEL 0 0
      %9 = [ %6 ]
      %10 = 0
      %11 = CMP %9 == %10
      %12 = NOT %11
      %13 COND %12 ==> %26 %14
      %14 LABEL 0 0
      %15 = [ %3 ]
      %16 = [ %5 ]
      %17 = OP2 %15 + %16
      %18 = [ %5 ]
      %19 [ %3 ] <- %18
      %20 [ %5 ] <- %17
      %21 = [ %6 ]
      %22 = 1
      %23 = OP2 %21 - %22
      %24 [ %6 ] <- %23
      %25 JUMP ==> %8
      %26 LABEL 0 1
      %27 = [ %5 ]
      %28 PUT 0 %27
      %29 RET
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_fib_tailcall() {
  let mut out = String::new();

  test(&mut out, "
    fun aux(a, b, n) {
      if (n == 0) {
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
      === fun aux ===
      %0 LABEL 3 0
      %1 = POP 0
      %2 = POP 1
      %3 = POP 2
      %4 = 0
      %5 = CMP %3 == %4
      %6 COND %5 ==> %7 %16
      %7 LABEL 0 3
      %8 = OP2 %1 + %2
      %9 = 1
      %10 = OP2 %3 - %9
      %11 = CONST aux
      %12 PUT 0 %2
      %13 PUT 1 %8
      %14 PUT 2 %10
      %15 TAIL-CALL %11
      %16 LABEL 0 1
      %17 PUT 0 %2
      %18 RET
      === fun fib ===
      %19 LABEL 1 3
      %20 = POP 0
      %21 = 1
      %22 = 0
      %23 = CONST aux
      %24 PUT 0 %21
      %25 PUT 1 %22
      %26 PUT 2 %20
      %27 TAIL-CALL %23
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_fib_call() {
  let mut out = String::new();

  test(&mut out, "
    ; TODO elif
    fun fib(n) {
      if (n == 0) {
        0
      } else {
        if (n == 1) {
          1
        } else {
          fib(n - 1) + fib(n - 2)
        }
      }
    }
  ");

  expect![[r#"
      === fun fib ===
      %0 LABEL 1 0
      %1 = POP 0
      %2 = 0
      %3 = CMP %1 == %2
      %4 COND %3 ==> %5 %31
      %5 LABEL 0 0
      %6 = 1
      %7 = CMP %1 == %6
      %8 COND %7 ==> %9 %27
      %9 LABEL 0 1
      %10 = 1
      %11 = OP2 %1 - %10
      %12 = CONST fib
      %13 PUT 0 %11
      %14 CALL %12 ==> %15
      %15 LABEL 1 1
      %16 = POP 0
      %17 = 2
      %18 = OP2 %1 - %17
      %19 = CONST fib
      %20 PUT 0 %18
      %21 CALL %19 ==> %22
      %22 LABEL 1 1
      %23 = POP 0
      %24 = OP2 %16 + %23
      %25 PUT 0 %24
      %26 RET
      %27 LABEL 0 1
      %28 = 1
      %29 PUT 0 %28
      %30 RET
      %31 LABEL 0 1
      %32 = 0
      %33 PUT 0 %32
      %34 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
