type Symbol = usize;
type Field = usize;
type Local = usize;

pub enum Item {
  Fun(Box<Fun>),
}

pub struct Fun {
  pub name: Symbol,
  pub args: Box<[Binding]>,
  pub body: Box<[Stmt]>,
}

// TODO: add optional type ascription

pub struct Binding {
  pub name: Option<Symbol>,
}

pub enum Expr {
  ConstBool(bool),                               // True
  ConstI64(i64),                                 // 3
  LogAnd(Box<(Expr, Expr)>),                     // a and b
  LogOr(Box<(Expr, Expr)>),                      // a or b
  Op1(Box<(Op1, Expr)>),                         // - a
  Op2(Box<(Op1, Expr, Expr)>),                   // a + b
  GetField(Box<(Expr, Field)>),                  // a.foo
  GetIndex(Box<(Expr, Expr)>),                   // a[b]
  PreLocal(Box<(Op1, Local)>),                   // ++ x
  PreField(Box<(Op1, Expr, Field)>),             // ++ a.foo
  PreIndex(Box<(Op1, Expr, Expr)>),              // ++ a[b]
  PostLocal(Box<(Local, Op1)>),                  // x ++
  PostField(Box<(Expr, Field, Op1)>),            // a.foo ++
  PostIndex(Box<(Expr, Expr, Op1)>),             // a[b] ++
  If(Box<(Expr, Box<[Stmt]>)>),                  // if a { ... }
  IfElse(Box<(Expr, Box<[Stmt]>, Box<[Stmt]>)>), // if a { ... } else { ... }
  Ternary(Box<(Expr, Expr, Expr)>),              // a ? b : c
  Var(Symbol),                                   // x
  Call(Box<(Expr, Box<[Expr]>)>),                // a(b, c)
  Loop(Box<[Stmt]>),                             // loop { ... }
  Fail,
}

pub enum Stmt {
  ExprList(Box<[Expr]>),                           // a, b
  Let(Box<(Box<[Binding]>, Box<[Expr]>)>),         // let x, y = a, b
  DefLocal(Box<(Local, Expr)>),                    // let mutable x = a
  SetLocal(Box<(Local, Expr)>),                    // x = 1
  SetField(Box<(Expr, Field, Expr)>),              // a.foo = 1
  SetIndex(Box<(Expr, Expr, Expr)>),               // a[b] = c
  SetLocalCompound(Box<(Local, Op2, Expr)>),       // x += a
  SetFieldCompound(Box<(Expr, Field, Op2, Expr)>), // a.foo += b
  SetIndexCompound(Box<(Expr, Expr, Op2, Expr)>),  // a[b] += c
  Break(Box<[Expr]>),                              // break a, b
  While(Box<(Expr, Box<[Stmt]>)>),                 // while (a) { ... }
  Continue,                                        // continue
  Ret(Box<[Expr]>),                                // return a, b
}

#[derive(Clone, Copy)]
pub enum Op1 {
  Dec, // x -- OR -- x
  Inc, // x ++ OR ++ x
  Neg, // - a
  Not, // ! a
}

#[derive(Clone, Copy)]
pub enum Op2 {
  Add,    // a + b
  BitAnd, // a & b
  BitOr,  // a | b
  BitXor, // a ^ b
  CmpEq,  // a == b
  CmpGe,  // a >= b
  CmpGt,  // a > b
  CmpLe,  // a <= b
  CmpLt,  // a < b
  CmpNe,  // a != b
  Div,    // a / b
  Mul,    // a * b
  Rem,    // a % b
  Shl,    // a << b
  Shr,    // a >> b
  Sub,    // a - b
}
