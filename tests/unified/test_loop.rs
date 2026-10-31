#![allow(unused_must_use)]

use crate::util;
use expect_test::expect;

#[test]
fn test_fib_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(n) {
      let mut x = 0
      let mut i = 0
      loop {
        if ! (i < n) { return x }
        x += i
        i += 1
      }
    }
  ");

  expect![[r#"
      %0 LABEL 1
      %1 = ARG 0
      %2 = 0
      %3 = DEF-LOCAL %2
      %4 = 0
      %5 = DEF-LOCAL %4
      %6 ==> GOTO %7
      %7 LABEL 0
      %8 = [ %5 ]
      %9 = %8 < %1
      %10 = ! %9
      %11 COND %10
      %12 ==> GOTO %18
      %13 ==> GOTO %14
      %14 LABEL 0
      %15 = [ %3 ]
      %16 PUT 0 %15
      %17 RET
      %18 LABEL 0
      %19 = [ %5 ]
      %20 = [ %3 ]
      %21 = %20 + %19
      %22 [ %3 ] <- %21
      %23 = 1
      %24 = [ %5 ]
      %25 = %24 + %23
      %26 [ %5 ] <- %25
      %27 ==> GOTO %7
  "#]].assert_eq(out.drain(..).as_ref());
}
