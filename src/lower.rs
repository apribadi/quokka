//! lowering pass
//!
//! abstract syntax tree -> linearized bytecode + item references

use crate::prelude::*;
use crate::ast;
use crate::ssa_0 as ssa;

type Symbol = usize;

// TODO: consider special lowering for arguments to cond

pub fn compile(item_list: &[ast::Item]) -> ssa::Module {
  let mut ctx = Ctx::new();
  let mut out = Out::new();

  /*
  for ast::Item::Fun(f) in item_list.iter() {
    let pos = out.code.len();
    scope_enter(&mut ctx.scope);
    let _ = out.emit(Inst::Label(f.args.len() as u32));

    for (i, x) in enumerate(f.args.iter()) {
      let y = out.emit(Inst::Get(i));
      if let Some(x) = x.name {
        scope_add(x, Referent::Value(y), &mut ctx.scope);
      }
    }

    compile_block_tail(f.body, &mut ctx, &mut out);
    scope_leave(&mut ctx.scope);
    out.decl.push(Fun { name: f.name, pos, len: out.code.len() - pos });
  }

*/
  ssa::Module {
    fun_defs: out.fun_defs.into_boxed_slice(),
    code: out.code.into_boxed_slice(),
  }
}

enum What {
  Points(Box<[Point]>),
  Values(Box<[usize]>),
}

// patch point
struct Point {
  index: usize,
  arity: Option<usize>,
}

enum Referent {
  Local(usize),
  Value(usize),
}

enum LoopInfo {
  TopLevel,
  Tail { label: Label },
  NonTail { label: Label, n_breaks: usize },
}

#[derive(Clone, Copy)]
struct Label {
  index: usize,
  arity: usize,
}

struct Ctx {
  scope: Scope,
  loops: LoopStack,
  // values: Buf<u32>,
  // points: Buf<Point>,
}

impl Ctx {
  fn new() -> Self {
    Self {
      scope: Scope::new(),
      loops: LoopStack::new(),
      // values: Buf::new(),
      // points: Buf::new(),
    }
  }
}

struct Scope {
  table: HashMap<Symbol, Referent>,
  level: usize,
  undo: Vec<(usize, Symbol, Option<Referent>)>,
}

impl Scope {
  fn new() -> Self {
    Self {
      table: HashMap::new(),
      level: 0,
      undo: Vec::new(),
    }
  }
}

fn scope_enter(t: &mut Scope) {
  t.level += 1;
}

fn scope_leave(t: &mut Scope) {
  while let Some((_, s, x)) = t.undo.pop_if(|&mut (n, _, _)| n == t.level) {
    if let Some(x) = x {
      t.table.insert(s, x);
    } else {
      t.table.remove(&s);
    }
  }
  t.level -= 1;
}

fn scope_add(s: Symbol, x: Referent, t: &mut Scope) {
  t.undo.push((t.level, s, t.table.insert(s, x)));
}

fn scope_get(s: Symbol, t: &Scope) -> Option<&Referent> {
  t.table.get(&s)
}

struct LoopStack {
  info: Vec<LoopInfo>,
  breaks: Vec<Point>,
}

impl LoopStack {
  fn new() -> Self {
    Self {
      info: Vec::from([LoopInfo::TopLevel]),
      breaks: Vec::new(),
    }
  }
}

/*
fn push_loop(a: Label, t: &mut LoopStack) {
  t.info.push(LoopInfo::NonTail { label: a, n_breaks: 0 });
}

fn pop_loop(t: &mut LoopStack, points: &mut Buf<Point>) -> u32 {
  let LoopInfo::NonTail { n_breaks, .. } = t.info.pop_one() else { unreachable!() };
  for p in t.breaks.pop_seq(n_breaks) {
    points.push(p);
  }
  return n_breaks;
}

fn push_loop_tail(a: Label, t: &mut LoopStack) {
  t.info.push(LoopInfo::Tail { label: a });
}

fn pop_loop_tail(t: &mut LoopStack) {
  let _ = t.info.pop_one();
}

*/
struct Out {
  fun_defs: Vec<ssa::FunDef>,
  code: Vec<ssa::Inst>,
}

impl Out {
  fn new() -> Self {
    Self {
      fun_defs: Vec::new(),
      code: Vec::new(),
    }
  }

  /*
  fn emit(&mut self, inst: Inst) -> u32 {
    let n = self.code.len();
    self.code.push(inst);
    return n;
  }

  fn emit_point(&mut self, arity: Option<u32>) -> Point {
    let i = self.emit(Inst::Goto(u32::MAX));
    return Point { index: i, arity };
  }

  fn emit_label(&mut self, arity: u32, ps: impl IntoIterator<Item = Point>) -> Label {
    let a = self.emit(Inst::Label(arity));
    let a = Label { index: a, arity };
    patch_point_list(a, ps, self);
    return a;
  }
  */
}
/*

fn patch_point_list(a: Label, ps: impl IntoIterator<Item = Point>, out: &mut Out) {
  for i in ps {
    if let Some(n) = i.arity && n != a.arity {
      // error, arity mismatch
      out.code[i.index] = Inst::GotoStaticError;
    } else {
      out.code[i.index] = Inst::Goto(a.index);
    }
  }
}

impl What {
  const NEVER: Self = What::NumPoints(0);

  const NIL: Self = What::NumValues(0);

  fn into_nil(self, ctx: &mut Ctx, out: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = out.emit_label(0, ctx.points.pop_seq(n_points));
      }
      What::NumValues(n_values) => {
        if n_values != 0 {
          // error, arity mismatch
          let _ = ctx.values.pop_seq(n_values);
          let _ = out.emit(Inst::GotoStaticError);
          let _ = out.emit(Inst::Label(0));
        }
      }
    }
  }

  fn into_value(self, ctx: &mut Ctx, out: &mut Out) -> u32 {
    match self {
      What::NumPoints(n_points) => {
        let _ = out.emit_label(1, ctx.points.pop_seq(n_points));
        let x = out.emit(Inst::Get(0));
        return x;
      }
      What::NumValues(n_values) => {
        if n_values == 1 {
          return ctx.values.pop_one();
        } else {
          // error, arity mismatch
          let _ = ctx.values.pop_seq(n_values);
          let _ = out.emit(Inst::GotoStaticError);
          let _ = out.emit(Inst::Label(1));
          let x = out.emit(Inst::Get(0));
          return x;
        }
      }
    }
  }

  fn into_value_list(self, arity: u32, ctx: &mut Ctx, out: &mut Out) {
    match self {
      What::NumPoints(n_points) => {
        let _ = out.emit_label(arity, ctx.points.pop_seq(n_points));
        for i in 0 .. arity {
          let x = out.emit(Inst::Get(i));
          ctx.values.push(x);
        }
      }
      What::NumValues(n_values) => {
        if arity != n_values {
          // error, arity mismatch
          let _ = ctx.values.pop_seq(n_values);
          let _ = out.emit(Inst::GotoStaticError);
          let _ = out.emit(Inst::Label(arity));
          for i in 0 .. arity {
            let x = out.emit(Inst::Get(i));
            ctx.values.push(x);
          }
        }
      }
    }
  }

  fn into_point_list(self, ctx: &mut Ctx, out: &mut Out) -> u32 {
    match self {
      What::NumPoints(n_points) => {
        return n_points;
      }
      What::NumValues(n_values) => {
        for (i, x) in enumerate(ctx.values.pop_seq(n_values)) {
          let _ = out.emit(Inst::Put(i, x));
        }
        let p = out.emit_point(Some(n_values));
        ctx.points.push(p);
        return 1;
      }
    }
  }
}

fn compile_expr<'a>(x: &Expr<'a>, ctx: &mut Ctx, out: &mut Out) -> What {
  match *x {
    Expr::And(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let x = out.emit(Inst::ConstBool(false));
      let _ = out.emit(Inst::Put(0, x));
      let r = out.emit_point(Some(1));
      ctx.points.push(r);
      let _ = out.emit_label(0, [q]);
      let n = compile_expr(y, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(1 + n);
    }
    Expr::Bool(x) => {
      let x = out.emit(Inst::ConstBool(x));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::Call(&(ref f, ref xs)) => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.push(x);
      }
      let f = compile_expr(f, ctx, out).into_value(ctx, out); // NB: evaluate *after* args
      for (i, x) in enumerate(ctx.values.pop_seq(n)) {
        let _ = out.emit(Inst::Put(i, x));
      }
      let _ = out.emit(Inst::Call(f));
      let p = out.emit_point(None);
      ctx.points.push(p);
      return What::NumPoints(1);
    }
    Expr::Field(&(ref x, s)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Field(x, s));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::If(&(ref x, ref ys)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      ctx.points.push(p);
      let _ = out.emit_label(0, [q]);
      let n = compile_block(ys, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(1 + n);
    }
    Expr::IfElse(&(ref x, ref ys, ref zs)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let m = compile_block(zs, ctx, out).into_point_list(ctx, out);
      let _ = out.emit_label(0, [q]);
      let n = compile_block(ys, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(m + n);
    }
    Expr::Index(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Index(x, y));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::Int(n) => {
      let x = out.emit(Inst::ConstInt(n));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::Loop(xs) => {
      let p = out.emit_point(Some(0));
      let a = out.emit_label(0, [p]);
      push_loop(a, &mut ctx.loops);
      let m = compile_block(xs, ctx, out).into_point_list(ctx, out);
      patch_point_list(a, ctx.points.pop_seq(m), out);
      let n = pop_loop(&mut ctx.loops, &mut ctx.points);
      return What::NumPoints(n);
    }
    Expr::Op1(&(f, ref x)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Op1(f, x));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::Op2(&(f, ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Op2(f, x, y));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::Or(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let n = compile_expr(y, ctx, out).into_point_list(ctx, out);
      let _ = out.emit_label(0, [q]);
      let x = out.emit(Inst::ConstBool(true));
      let _ = out.emit(Inst::Put(0, x));
      let r = out.emit_point(Some(1));
      ctx.points.push(r);
      return What::NumPoints(n + 1);
    }
    Expr::PostOp(&(s, f)) => {
      if let Some(&Referent::Local(v)) = scope_get(s, &ctx.scope) {
        let x = out.emit(Inst::GetLocal(v));
        let y = out.emit(Inst::Op1(f, x));
        let _ = out.emit(Inst::SetLocal(v, y));
        ctx.values.push(x); // post
        return What::NumValues(1);
      }
      // error, post-op must target a local variable
      let _ = out.emit(Inst::GotoStaticError);
      let _ = out.emit(Inst::Label(1));
      let x = out.emit(Inst::Get(0));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::PreOp(&(s, f)) => {
      if let Some(&Referent::Local(v)) = scope_get(s, &ctx.scope) {
        let x = out.emit(Inst::GetLocal(v));
        let y = out.emit(Inst::Op1(f, x));
        let _ = out.emit(Inst::SetLocal(v, y));
        ctx.values.push(y); // pre
        return What::NumValues(1);
      }
      // error, pre-op must target a local variable
      let _ = out.emit(Inst::GotoStaticError);
      let _ = out.emit(Inst::Label(1));
      let x = out.emit(Inst::Get(0));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::Ternary(&(ref x, ref y, ref z)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let m = compile_expr(z, ctx, out).into_point_list(ctx, out);
      let _ = out.emit_label(0, [q]);
      let n = compile_expr(y, ctx, out).into_point_list(ctx, out);
      return What::NumPoints(m + n);
    }
    Expr::Undefined => {
      // error, evaluating undefined expression
      let _ = out.emit(Inst::GotoStaticError);
      let _ = out.emit(Inst::Label(1));
      let x = out.emit(Inst::Get(0));
      ctx.values.push(x);
      return What::NumValues(1);
    }
    Expr::Variable(s) => {
      match scope_get(s, &ctx.scope) {
        None => {
          let x = out.emit(Inst::Const(s));
          ctx.values.push(x);
        }
        Some(&Referent::Value(x)) => {
          ctx.values.push(x);
        }
        Some(&Referent::Local(v)) => {
          let x = out.emit(Inst::GetLocal(v));
          ctx.values.push(x);
        }
      }
      return What::NumValues(1);
    }
  }
}

fn compile_expr_tail<'a>(x: &Expr<'a>, ctx: &mut Ctx, out: &mut Out) {
  match *x {
    Expr::And(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let x = out.emit(Inst::ConstBool(false));
      let _ = out.emit(Inst::Put(0, x));
      let _ = out.emit(Inst::Ret);
      let _ = out.emit_label(0, [q]);
      compile_expr_tail(y, ctx, out);
    }
    Expr::Call(&(ref f, ref xs)) => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.push(x);
      }
      let f = compile_expr(f, ctx, out).into_value(ctx, out); // NB: evaluate *after* args
      for (i, x) in enumerate(ctx.values.pop_seq(n)) {
        let _ = out.emit(Inst::Put(i, x));
      }
      let _ = out.emit(Inst::TailCall(f));
    }
    Expr::If(&(ref x, ref ys)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      let _ = out.emit(Inst::Ret);
      let _ = out.emit_label(0, [q]);
      compile_block_tail(ys, ctx, out);
    }
    Expr::IfElse(&(ref x, ref ys, ref zs)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      compile_block_tail(zs, ctx, out);
      let _ = out.emit_label(0, [q]);
      compile_block_tail(ys, ctx, out);
    }
    Expr::Loop(xs) => {
      let p = out.emit_point(Some(0));
      let a = out.emit_label(0, [p]);
      push_loop_tail(a, &mut ctx.loops);
      let n = compile_block(xs, ctx, out).into_point_list(ctx, out);
      patch_point_list(a, ctx.points.pop_seq(n), out);
      pop_loop_tail(&mut ctx.loops);
    }
    Expr::Or(&(ref x, ref y)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      compile_expr_tail(y, ctx, out);
      let _ = out.emit_label(0, [q]);
      let x = out.emit(Inst::ConstBool(true));
      let _ = out.emit(Inst::Put(0, x));
      let _ = out.emit(Inst::Ret);
    }
    Expr::Ternary(&(ref x, ref y, ref z)) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let p = out.emit_point(Some(0));
      let q = out.emit_point(Some(0));
      let _ = out.emit_label(0, [p]);
      compile_expr_tail(z, ctx, out);
      let _ = out.emit_label(0, [q]);
      compile_expr_tail(y, ctx, out);
    }
    Expr::Bool(..)
    | Expr::Field(..)
    | Expr::Index(..)
    | Expr::Int(..)
    | Expr::Op1(..)
    | Expr::Op2(..)
    | Expr::PostOp(..)
    | Expr::PreOp(..)
    | Expr::Undefined
    | Expr::Variable(..) => {
      let What::NumValues(1) = compile_expr(x, ctx, out) else { unreachable!() };
      let _ = out.emit(Inst::Put(0, ctx.values.pop_one()));
      let _ = out.emit(Inst::Ret);
    }
  }
}

fn compile_stmt<'a>(x: &Stmt<'a>, ctx: &mut Ctx, out: &mut Out) -> What {
  match *x {
    Stmt::ExprList(xs) => {
      return compile_expr_list(xs, ctx, out);
    }
    Stmt::Break(xs) => {
      match ctx.loops.info.top() {
        LoopInfo::TopLevel => {
          // error, break is not inside loop
          let _ = out.emit(Inst::GotoStaticError);
        }
        LoopInfo::NonTail { .. } => {
          let n = compile_expr_list(xs, ctx, out).into_point_list(ctx, out);
          let &mut LoopInfo::NonTail { ref mut n_breaks, .. } = ctx.loops.info.top_mut() else { unreachable!() };
          for p in ctx.points.pop_seq(n) {
            ctx.loops.breaks.push(p);
            *n_breaks += 1;
          }
        }
        LoopInfo::Tail { .. } => {
          compile_expr_list_tail(xs, ctx, out);
        }
      }
      return What::NEVER;
    }
    Stmt::Continue => {
      match ctx.loops.info.top() {
        LoopInfo::TopLevel => {
          // error, break is not inside loop
          let _ = out.emit(Inst::GotoStaticError);
        }
        LoopInfo::NonTail { label, .. } | LoopInfo::Tail { label } => {
          // NB: all loop headers have arity zero
          let _ = out.emit(Inst::Goto(label.index));
        }
      }
      return What::NEVER;
    }
    Stmt::Let(xs, ys) => {
      let n = xs.len() as u32;
      // NB: we do the bindings from left to right, so later bindings shadow
      // earlier ones.
      compile_expr_list(ys, ctx, out).into_value_list(n, ctx, out);
      for (x, y) in zip(xs, ctx.values.pop_seq(n)) {
        if let Some(x) = x.name {
          scope_add(x, Referent::Value(y), &mut ctx.scope);
        }
      }
      return What::NIL;
    }
    Stmt::Return(xs) => {
      compile_expr_list_tail(xs, ctx, out);
      return What::NEVER;
    }
    Stmt::Set(s, ref x) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      if let Some(&Referent::Local(v)) = scope_get(s, &ctx.scope) {
        let _ = out.emit(Inst::SetLocal(v, x));
      } else {
        // error, symbol does not refer to local variable
        let _ = out.emit(Inst::GotoStaticError);
        let _ = out.emit(Inst::Label(0));
      }
      return What::NIL;
    }
    Stmt::SetField(ref x, s, ref y) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::SetField(x, s, y));
      return What::NIL;
    }
    Stmt::SetIndex(ref x, ref y, ref z) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let y = compile_expr(y, ctx, out).into_value(ctx, out);
      let z = compile_expr(z, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::SetIndex(x, y, z));
      return What::NIL;
    }
    Stmt::Var(s, ref x) => {
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let x = out.emit(Inst::Local(x));
      scope_add(s, Referent::Local(x), &mut ctx.scope);
      return What::NIL;
    }
    Stmt::While(ref x, ys) => {
      let p = out.emit_point(Some(0));
      let a = out.emit_label(0, [p]);
      push_loop(a, &mut ctx.loops);
      let x = compile_expr(x, ctx, out).into_value(ctx, out);
      let _ = out.emit(Inst::Cond(x));
      let q = out.emit_point(Some(0));
      ctx.points.push(q);
      let r = out.emit_point(Some(0));
      let _ = out.emit_label(0, [r]);
      let m = compile_block(ys, ctx, out).into_point_list(ctx, out);
      patch_point_list(a, ctx.points.pop_seq(m), out);
      let n = pop_loop(&mut ctx.loops, &mut ctx.points);
      return What::NumPoints(1 + n);
    }
  }
}

fn compile_stmt_tail<'a>(x: &Stmt<'a>, ctx: &mut Ctx, out: &mut Out) {
  match *x {
    Stmt::ExprList(xs) => {
      compile_expr_list_tail(xs, ctx, out);
    }
    | Stmt::Break(..)
    | Stmt::Continue
    | Stmt::Return(..) => {
      let What::NumPoints(0) = compile_stmt(x, ctx, out) else { unreachable!() };
    }
    Stmt::Let(..)
    | Stmt::Set(..)
    | Stmt::SetField(..)
    | Stmt::SetIndex(..)
    | Stmt::Var(..)
    | Stmt::While(..) => {
      let What::NumValues(0) = compile_stmt(x, ctx, out) else { unreachable!() };
      let _ = out.emit(Inst::Ret);
    }
  }
}

fn compile_block<'a>(xs: &'a [Stmt<'a>], ctx: &mut Ctx, out: &mut Out) -> What {
  match xs.split_last() {
    None => {
      return What::NIL;
    }
    Some((y, xs)) => {
      scope_enter(&mut ctx.scope);
      for x in xs {
        compile_stmt(x, ctx, out).into_nil(ctx, out);
      }
      let w = compile_stmt(y, ctx, out);
      scope_leave(&mut ctx.scope);
      return w;
    }
  }
}

fn compile_block_tail<'a>(xs: &'a [Stmt<'a>], ctx: &mut Ctx, out: &mut Out) {
  match xs.split_last() {
    None => {
      let _ = out.emit(Inst::Ret);
    }
    Some((y, xs)) => {
      scope_enter(&mut ctx.scope);
      for x in xs {
        compile_stmt(x, ctx, out).into_nil(ctx, out);
      }
      compile_stmt_tail(y, ctx, out);
      scope_leave(&mut ctx.scope);
    }
  }
}

fn compile_expr_list<'a>(xs: &'a [Expr<'a>], ctx: &mut Ctx, out: &mut Out) -> What {
  match xs {
    [x] => {
      return compile_expr(x, ctx, out);
    }
    xs => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.push(x);
      }
      return What::NumValues(n);
    }
  }
}

fn compile_expr_list_tail<'a>(xs: &'a [Expr<'a>], ctx: &mut Ctx, out: &mut Out) {
  match xs {
    [x] => {
      compile_expr_tail(x, ctx, out);
    }
    xs => {
      let n = xs.len() as u32;
      for x in xs.iter() {
        let x = compile_expr(x, ctx, out).into_value(ctx, out);
        ctx.values.push(x);
      }
      for (i, x) in enumerate(ctx.values.pop_seq(n)) {
        let _ = out.emit(Inst::Put(i, x));
      }
      let _ = out.emit(Inst::Ret);
    }
  }
}
*/
