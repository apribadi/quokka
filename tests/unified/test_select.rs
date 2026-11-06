#![allow(unused_must_use)]

use crate::util::dump;
use expect_test::expect;

#[test]
fn test_projection() {
  let mut out = String::new();

  dump(&mut out, "
    fun select(p, x, y) {
      p ? x : y
    }

    fun foo(p, q) {
      p ? 1 + 1 + 1 : q ? 1 + 1 : 1
    }
  ");

  expect![[r#"
      === fun select ===
      %0 LABEL 3 0
      %1 = POP 0
      %2 = POP 1
      %3 = POP 2
      %4 COND %1
      %5 ==> GOTO %7
      %6 ==> GOTO %10
      %7 LABEL 0 1
      %8 PUT 0 %3
      %9 RET
      %10 LABEL 0 1
      %11 PUT 0 %2
      %12 RET
      === fun foo ===
      %13 LABEL 2 0
      %14 = POP 0
      %15 = POP 1
      %16 COND %14
      %17 ==> GOTO %19
      %18 ==> GOTO %33
      %19 LABEL 0 0
      %20 COND %15
      %21 ==> GOTO %23
      %22 ==> GOTO %27
      %23 LABEL 0 1
      %24 = 1
      %25 PUT 0 %24
      %26 RET
      %27 LABEL 0 1
      %28 = 1
      %29 = 1
      %30 = %28 + %29
      %31 PUT 0 %30
      %32 RET
      %33 LABEL 0 1
      %34 = 1
      %35 = 1
      %36 = %34 + %35
      %37 = 1
      %38 = %36 + %37
      %39 PUT 0 %38
      %40 RET

      SIMPLIFIED

      === fun select ===
      %0 LABEL 3 0
      %1 = POP 0
      %2 = POP 1
      %3 = POP 2
      %4 COND %1
      %5 ==> GOTO %7
      %6 ==> GOTO %10
      %7 LABEL 0 1
      %8 PUT 0 %3
      %9 RET
      %10 LABEL 0 1
      %11 PUT 0 %2
      %12 RET
      === fun foo ===
      %13 LABEL 2 0
      %14 = POP 0
      %15 = POP 1
      %16 COND %14
      %17 ==> GOTO %19
      %18 ==> GOTO %33
      %19 LABEL 0 0
      %20 COND %15
      %21 ==> GOTO %23
      %22 ==> GOTO %27
      %23 LABEL 0 1
      %24 = 1
      %25 PUT 0 %24
      %26 RET
      %27 LABEL 0 1
      %28 = 1
      %29 = 1
      %30 = %28 + %29
      %31 PUT 0 %30
      %32 RET
      %33 LABEL 0 1
      %34 = 1
      %35 = 1
      %36 = %34 + %35
      %37 = 1
      %38 = %36 + %37
      %39 PUT 0 %38
      %40 RET
  "#]].assert_eq(out.drain(..).as_ref());
}
