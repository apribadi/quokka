use crate::util;
use expect_test::expect;

#[test]
fn test_fib_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun fib(n) {
      let mutable a = 1
      let mutable b = 0
      let mutable n = n
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
      [
          Fun(
              Fun {
                  name: Some(
                      "fib",
                  ),
                  args: [
                      Bind {
                          name: Some(
                              "n",
                          ),
                      },
                  ],
                  body: [
                      DefLocal(
                          Some(
                              "a",
                          ),
                          LiteralI64(
                              1,
                          ),
                      ),
                      DefLocal(
                          Some(
                              "b",
                          ),
                          LiteralI64(
                              0,
                          ),
                      ),
                      DefLocal(
                          Some(
                              "n",
                          ),
                          Variable(
                              "n",
                          ),
                      ),
                      ExprList(
                          [
                              Loop(
                                  [
                                      ExprList(
                                          [
                                              If(
                                                  (
                                                      Op2(
                                                          (
                                                              CmpEq,
                                                              Variable(
                                                                  "n",
                                                              ),
                                                              LiteralI64(
                                                                  0,
                                                              ),
                                                          ),
                                                      ),
                                                      [
                                                          Ret(
                                                              [
                                                                  Variable(
                                                                      "b",
                                                                  ),
                                                              ],
                                                          ),
                                                      ],
                                                  ),
                                              ),
                                          ],
                                      ),
                                      Let(
                                          [
                                              Bind {
                                                  name: Some(
                                                      "c",
                                                  ),
                                              },
                                          ],
                                          [
                                              Op2(
                                                  (
                                                      Add,
                                                      Variable(
                                                          "a",
                                                      ),
                                                      Variable(
                                                          "b",
                                                      ),
                                                  ),
                                              ),
                                          ],
                                      ),
                                      SetLocal(
                                          "a",
                                          Variable(
                                              "b",
                                          ),
                                      ),
                                      SetLocal(
                                          "b",
                                          Variable(
                                              "c",
                                          ),
                                      ),
                                      SetLocal(
                                          "n",
                                          Op2(
                                              (
                                                  Sub,
                                                  Variable(
                                                      "n",
                                                  ),
                                                  LiteralI64(
                                                      1,
                                                  ),
                                              ),
                                          ),
                                      ),
                                  ],
                              ),
                          ],
                      ),
                  ],
              },
          ),
      ]
  "#]].assert_eq(out.drain(..).as_ref());
}

#[test]
fn test_fib_tailcall() {
  let mut out = String::new();

  util::dump(&mut out, "
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
      [
          Fun(
              Fun {
                  name: Some(
                      "aux",
                  ),
                  args: [
                      Bind {
                          name: Some(
                              "a",
                          ),
                      },
                      Bind {
                          name: Some(
                              "b",
                          ),
                      },
                      Bind {
                          name: Some(
                              "n",
                          ),
                      },
                  ],
                  body: [
                      ExprList(
                          [
                              IfElse(
                                  (
                                      Op2(
                                          (
                                              CmpEq,
                                              Variable(
                                                  "n",
                                              ),
                                              LiteralI64(
                                                  0,
                                              ),
                                          ),
                                      ),
                                      [
                                          ExprList(
                                              [
                                                  Variable(
                                                      "b",
                                                  ),
                                              ],
                                          ),
                                      ],
                                      [
                                          ExprList(
                                              [
                                                  Call(
                                                      (
                                                          Variable(
                                                              "aux",
                                                          ),
                                                          [
                                                              Variable(
                                                                  "b",
                                                              ),
                                                              Op2(
                                                                  (
                                                                      Add,
                                                                      Variable(
                                                                          "a",
                                                                      ),
                                                                      Variable(
                                                                          "b",
                                                                      ),
                                                                  ),
                                                              ),
                                                              Op2(
                                                                  (
                                                                      Sub,
                                                                      Variable(
                                                                          "n",
                                                                      ),
                                                                      LiteralI64(
                                                                          1,
                                                                      ),
                                                                  ),
                                                              ),
                                                          ],
                                                      ),
                                                  ),
                                              ],
                                          ),
                                      ],
                                  ),
                              ),
                          ],
                      ),
                  ],
              },
          ),
          Fun(
              Fun {
                  name: Some(
                      "fib",
                  ),
                  args: [
                      Bind {
                          name: Some(
                              "n",
                          ),
                      },
                  ],
                  body: [
                      ExprList(
                          [
                              Call(
                                  (
                                      Variable(
                                          "aux",
                                      ),
                                      [
                                          LiteralI64(
                                              1,
                                          ),
                                          LiteralI64(
                                              0,
                                          ),
                                          Variable(
                                              "n",
                                          ),
                                      ],
                                  ),
                              ),
                          ],
                      ),
                  ],
              },
          ),
      ]
  "#]].assert_eq(out.drain(..).as_ref());
}
