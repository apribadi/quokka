use crate::util;
use expect_test::expect;

#[test]
fn test_fib_loop() {
  let mut out = String::new();

  util::dump(&mut out, "
    fun foo(n) {
      let mutable x = 0
      let mutable i = 0
      loop {
        if ! (i < n) { return x }
        x += i
        i += 1
      }
    }
  ");

  expect![[r#"
      [
          Fun(
              Fun {
                  name: Some(
                      "foo",
                  ),
                  args: [
                      Binding {
                          name: Some(
                              "n",
                          ),
                      },
                  ],
                  body: [
                      DefLocal(
                          Some(
                              "x",
                          ),
                          ConstI64(
                              0,
                          ),
                      ),
                      DefLocal(
                          Some(
                              "i",
                          ),
                          ConstI64(
                              0,
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
                                                      Op1(
                                                          (
                                                              Not,
                                                              Op2(
                                                                  (
                                                                      CmpLt,
                                                                      Variable(
                                                                          "i",
                                                                      ),
                                                                      Variable(
                                                                          "n",
                                                                      ),
                                                                  ),
                                                              ),
                                                          ),
                                                      ),
                                                      [
                                                          Ret(
                                                              [
                                                                  Variable(
                                                                      "x",
                                                                  ),
                                                              ],
                                                          ),
                                                      ],
                                                  ),
                                              ),
                                          ],
                                      ),
                                      SetLocalCompound(
                                          "x",
                                          Add,
                                          Variable(
                                              "i",
                                          ),
                                      ),
                                      SetLocalCompound(
                                          "i",
                                          Add,
                                          ConstI64(
                                              1,
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
                      Binding {
                          name: Some(
                              "a",
                          ),
                      },
                      Binding {
                          name: Some(
                              "b",
                          ),
                      },
                      Binding {
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
                                              ConstI64(
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
                                                                      ConstI64(
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
                      Binding {
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
                                          ConstI64(
                                              1,
                                          ),
                                          ConstI64(
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
