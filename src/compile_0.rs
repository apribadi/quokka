//! lowering pass
//!
//! abstract syntax tree -> linearized bytecode + item references

#![allow(warnings)]

use crate::prelude::*;
use crate::ast;
use crate::ast::Expr;
use crate::ssa_0 as ssa;
use crate::ssa_0::Inst;
use crate::symbol::Symbol;
use crate::symbol::SymbolTable;
use std::mem::replace;

type Value = usize;
type Local = usize;

#[derive(Clone, Copy)]
enum Effect {
  NumPoints(usize),
  NumValues(usize),
}

#[derive(Clone, Copy)]
enum Referent {
  Local(Local),
  Value(Value),
}

#[derive(Clone, Copy)]
struct Point {
  index: usize,
  arity: Option<usize>,
}

#[derive(Clone, Copy)]
struct Label {
  index: usize,
  arity: usize,
}

enum LoopKind {
  TopLevel,
  NonTail { head: Label, break_count: usize },
  Tail { head: Label },
}

struct Ctx<'a> {
  stack: StackCtx,
  loops: LoopsCtx,
  scope: ScopeCtx<'a>,
  symbol_table: SymbolTable,
  out: Out,
}

struct StackCtx {
  value: Vec<Value>,
  point: Vec<Point>,
}

struct LoopsCtx {
  kind: LoopKind,
  stack: Vec<LoopKind>,
  break_stack: Vec<Point>,
}

struct ScopeCtx<'a> {
  table: HashMap<&'a str, Referent>,
  stack: Vec<usize>,
  entry_count: usize,
  entry: Vec<(&'a str, Option<Referent>)>,
}

struct Out {
  block_label: usize,
  code: Vec<ssa::Inst>,
}

impl Effect {
  const NEVER: Self = Effect::NumPoints(0);

  const NIL: Self = Effect::NumValues(0);

  fn into_nil(self, ctx: &mut Ctx) {
    match self {
      Effect::NumPoints(num_points) => {
        let a = ctx.out.emit_label(0);
        for p in ctx.stack.point.pop_list(num_points) {
          ctx.out.patch(p, a);
        }
      }
      Effect::NumValues(0) => {
      }
      Effect::NumValues(num_values) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(num_values);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(0);
      }
    }
  }

  fn into_value(self, ctx: &mut Ctx) -> Value {
    match self {
      Effect::NumPoints(num_points) => {
        let a = ctx.out.emit_label(1);
        for p in ctx.stack.point.pop_list(num_points) {
          ctx.out.patch(p, a);
        }
        ctx.out.emit(Inst::GetArg(0))
      }
      Effect::NumValues(1) => {
        ctx.stack.value.pop().unwrap()
      }
      Effect::NumValues(num_values) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(num_values);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(1);
        ctx.out.emit(Inst::GetArg(0))
      }
    }
  }

  fn into_value_list(self, arity: usize, ctx: &mut Ctx) {
    match self {
      Effect::NumPoints(num_points) => {
        let a = ctx.out.emit_label(arity);
        for p in ctx.stack.point.pop_list(num_points) {
          ctx.out.patch(p, a);
        }
        for i in 0 .. arity {
          ctx.stack.value.push(ctx.out.emit(Inst::GetArg(i)));
        }
      }
      Effect::NumValues(num_values) if arity == num_values => {
      }
      Effect::NumValues(num_values) => {
        // error, arity mismatch
        let _ = ctx.stack.value.pop_list(num_values);
        let _ = ctx.out.emit(Inst::Fail);
        let _ = ctx.out.emit_label(arity);
        for i in 0 .. arity {
          ctx.stack.value.push(ctx.out.emit(Inst::GetArg(i)));
        }
      }
    }
  }

  fn into_point_list(self, ctx: &mut Ctx) -> usize {
    match self {
      Effect::NumPoints(num_points) => {
        num_points
      }
      Effect::NumValues(num_values) => {
        for (i, x) in ctx.stack.value.pop_list(num_values).enumerate() {
          let _ = ctx.out.emit(Inst::SetOut(i, x));
        }
        // TODO: exity
        ctx.stack.point.push(ctx.out.emit_point(Some(num_values)));
        1
      }
    }
  }
}

impl LoopsCtx {
  fn enter(&mut self, head: Label) {
    self.stack.push(replace(&mut self.kind, LoopKind::NonTail { head, break_count: 0 }));
  }

  fn leave(&mut self, points: &mut Vec<Point>) -> usize {
    match replace(&mut self.kind, self.stack.pop().unwrap()) {
      LoopKind::NonTail { break_count, .. } => {
        for p in self.break_stack.pop_list(break_count) {
          points.push(p);
        }
        break_count
      }
      _ =>
        unreachable!(),
    }
  }

  fn enter_tail(&mut self, head: Label) {
    self.stack.push(replace(&mut self.kind, LoopKind::Tail { head }));
  }

  fn leave_tail(&mut self) {
    match replace(&mut self.kind, self.stack.pop().unwrap()) {
      LoopKind::Tail { .. } =>
        (),
      _ =>
        unreachable!(),
    }
  }
}

impl<'a> ScopeCtx<'a> {
  fn enter(&mut self) {
    let n = replace(&mut self.entry_count, 0);
    self.stack.push(n);
  }

  fn leave(&mut self) {
    let n = replace(&mut self.entry_count, self.stack.pop().unwrap());
    for (s, x) in self.entry.pop_list(n) {
      if let Some(x) = x {
        self.table.insert(s, x);
      } else {
        self.table.remove(&s);
      }
    }
  }

  fn bind(&mut self, s: &'a str, x: Referent) {
    let y = self.table.insert(s, x);
    self.entry_count += 1;
    self.entry.push((s, y));
  }

  fn lookup(&self, s: &'a str) -> Option<&Referent> {
    self.table.get(&s)
  }
}

impl Out {
  fn emit(&mut self, inst: Inst) -> usize {
    let i = self.code.len();
    self.code.push(inst);
    i
  }

  fn emit_point(&mut self, arity: Option<usize>) -> Point {
    let i = self.emit(Inst::Jump(usize::MAX));
    Point { index: i, arity }
  }

  fn emit_label(&mut self, arity: usize) -> Label {
    let i = self.emit(Inst::Label(arity, 0));
    self.block_label = i;
    Label { index: i, arity }
  }

  fn patch(&mut self, p: Point, a: Label) {
    if let Some(n) = p.arity && n != a.arity {
      self.code[p.index] = Inst::Fail; // arity mismatch
    } else {
      self.code[p.index] = Inst::Jump(a.index);
    }
  }
}

impl<'a> Ctx<'a> {
  fn new() -> Self {
    Self {
      stack: StackCtx {
        value: Vec::new(),
        point: Vec::new(),
      },
      loops: LoopsCtx {
        kind: LoopKind::TopLevel,
        stack: Vec::new(),
        break_stack: Vec::new(),
      },
      scope: ScopeCtx {
        table: HashMap::new(),
        stack: Vec::new(),
        entry_count: 0,
        entry: Vec::new(),
      },
      symbol_table: SymbolTable::new(),
      out: Out {
        block_label: 0,
        code: Vec::new(),
      },
    }
  }

  fn compile_expr(&mut self, x: &Expr<'a>) -> Effect {
    match *x {
      Expr::LiteralBool(x) => {
        self.stack.value.push(self.out.emit(Inst::ConstBool(x)));
        Effect::NumValues(1)
      }
      Expr::LiteralI64(x) => {
        self.stack.value.push(self.out.emit(Inst::ConstI64(x)));
        Effect::NumValues(1)
      }
      Expr::Variable(s) => {
        match self.scope.lookup(s) {
          None => {
            self.stack.value.push(self.out.emit(Inst::Const(self.symbol_table.intern(s))));
          }
          Some(&Referent::Local(v)) => {
            self.stack.value.push(self.out.emit(Inst::GetLocal(v)))
          }
          Some(&Referent::Value(x)) => {
            self.stack.value.push(x)
          }
        }
        Effect::NumValues(1)
      }
      Expr::Fail => {
        let _ = self.out.emit(Inst::Fail);
        Effect::NumPoints(0)
      }
      _ => {
        unimplemented!()
      }
    }
  }
  /*
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
    }
  }
*/
}
