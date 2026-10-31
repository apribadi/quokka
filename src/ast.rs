type Symbol<'a> = &'a str;

#[derive(Debug)]
pub enum Item<'a> {
  Fun(Box<Fun<'a>>),
}

#[derive(Debug)]
pub struct Fun<'a> {
  pub name: Option<Symbol<'a>>,
  pub args: Box<[Bind<'a>]>,
  pub body: Box<[Stmt<'a>]>,
}

#[derive(Debug)]
pub enum Stmt<'a> {
  ExprList(Box<[Expr<'a>]>),                   // a, b
  Let(Box<[Bind<'a>]>, Box<[Expr<'a>]>),       // let x, y = a, b
  DefLocal(Option<Symbol<'a>>, Expr<'a>),      // let mutable x = a
  SetLocal(Symbol<'a>, Expr<'a>),              // x = 1
  SetField(Expr<'a>, Symbol<'a>, Expr<'a>),    // a.foo = 1
  SetIndex(Expr<'a>, Expr<'a>, Expr<'a>),      // a[b] = c
  SetLocalCompound(Symbol<'a>, Op2, Expr<'a>), // x += a
  Break(Box<[Expr<'a>]>),                      // break a, b
  While(Expr<'a>, Box<[Stmt<'a>]>),            // while (a) { ... }
  Continue,                                    // continue
  Ret(Box<[Expr<'a>]>),                        // return a, b
}

#[derive(Debug)]
pub enum Expr<'a> {
  LiteralBool(bool),                                         // true OR false
  LiteralI64(i64),                                           // 3
  GetField(Box<(Expr<'a>, Symbol<'a>)>),                     // a.foo
  GetIndex(Box<(Expr<'a>, Expr<'a>)>),                       // a[b]
  If(Box<(Expr<'a>, Box<[Stmt<'a>]>)>),                      // if (a) { ... }
  IfElse(Box<(Expr<'a>, Box<[Stmt<'a>]>, Box<[Stmt<'a>]>)>), // if (a) { ... } else { ... }
  Op1(Box<(Op1, Expr<'a>)>),                                 // - a
  Op2(Box<(Op2, Expr<'a>, Expr<'a>)>),                       // a + b
  Select(Box<(Expr<'a>, Expr<'a>, Expr<'a>)>),               // a ? b : c
  ShortCircuitAnd(Box<(Expr<'a>, Expr<'a>)>),                // a && b
  ShortCircuitOr(Box<(Expr<'a>, Expr<'a>)>),                 // a || b
  Variable(Symbol<'a>),                                      // x
  Call(Box<(Expr<'a>, Box<[Expr<'a>]>)>),                    // a(b, c)
  Loop(Box<[Stmt<'a>]>),                                     // loop { ... }
  Fail,
}

// TODO: add optional type ascription

#[derive(Debug)]
pub struct Bind<'a> {
  pub name: Option<Symbol<'a>>,
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum Op1 {
  Neg, // - a
  Not, // ! a
}

#[derive(Clone, Copy)]
#[derive(Debug)]
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
