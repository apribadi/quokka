type Symbol<'a> = &'a str;

#[derive(Debug)]
pub enum Item<'a> {
  Fun(Box<Fun<'a>>),
}

#[derive(Debug)]
pub struct Fun<'a> {
  pub name: Symbol<'a>,
  pub args: Box<[Binding<'a>]>,
  pub body: Box<[Stmt<'a>]>,
}

// TODO: add optional type ascription

#[derive(Debug)]
pub struct Binding<'a> {
  pub name: Option<Symbol<'a>>,
}

#[derive(Debug)]
pub enum Expr<'a> {
  ConstBool(bool),                                           // true OR false
  ConstI64(i64),                                             // 3
  LogAnd(Box<(Expr<'a>, Expr<'a>)>),                         // a && b
  LogOr(Box<(Expr<'a>, Expr<'a>)>),                          // a || b
  Op1(Box<(Op1, Expr<'a>)>),                                 // - a
  Op2(Box<(Op2, Expr<'a>, Expr<'a>)>),                       // a + b
  GetField(Box<(Expr<'a>, Symbol<'a>)>),                     // a.foo
  GetIndex(Box<(Expr<'a>, Expr<'a>)>),                       // a[b]
  IncDecLocal(Box<(PrePost, IncDec, Symbol<'a>)>),           // ++ x OR x ++
  If(Box<(Expr<'a>, Box<[Stmt<'a>]>)>),                      // if (a) { ... }
  IfElse(Box<(Expr<'a>, Box<[Stmt<'a>]>, Box<[Stmt<'a>]>)>), // if (a) { ... } else { ... }
  Select(Box<(Expr<'a>, Expr<'a>, Expr<'a>)>),               // a ? b : c
  Var(Symbol<'a>),                                           // x
  Call(Box<(Expr<'a>, Box<[Expr<'a>]>)>),                    // a(b, c)
  Loop(Box<[Stmt<'a>]>),                                     // loop { ... }
  Fail,
}

#[derive(Debug)]
pub enum Stmt<'a> {
  ExprList(Box<[Expr<'a>]>),                          // a, b
  Let(Box<(Box<[Binding<'a>]>, Box<[Expr<'a>]>)>),    // let x, y = a, b
  DefLocal(Box<(Symbol<'a>, Expr<'a>)>),              // let mutable x = a
  SetLocal(Box<(Symbol<'a>, Expr<'a>)>),              // x = 1
  SetField(Box<(Expr<'a>, Symbol<'a>, Expr<'a>)>),    // a.foo = 1
  SetIndex(Box<(Expr<'a>, Expr<'a>, Expr<'a>)>),      // a[b] = c
  SetLocalCompound(Box<(Symbol<'a>, Op2, Expr<'a>)>), // x += a
  Break(Box<[Expr<'a>]>),                             // break a, b
  While(Box<(Expr<'a>, Box<[Stmt<'a>]>)>),            // while (a) { ... }
  Continue,                                           // continue
  Ret(Box<[Expr<'a>]>),                               // return a, b
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum IncDec {
  Inc, // ++
  Dec, // --
}

#[derive(Clone, Copy)]
#[derive(Debug)]
pub enum PrePost {
  Pre,
  Post,
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
