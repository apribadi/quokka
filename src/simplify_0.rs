//!
//!
//!

// #![allow(warnings)]

use crate::prelude::*;
use crate::ssa_0::Inst;
use crate::ssa_0::FunDef;
use crate::ssa_0::Module;
use crate::ssa_0::iter_block_insts;

#[derive(Clone)]
enum Remap {
  None,
  Value(usize),
  Alias(usize),
}

fn resolve(remap: &Vec<Remap>, i: usize) -> Option<usize> {
  let mut i = i;
  loop {
    match &remap[i] {
      &Remap::None => { return None },
      &Remap::Value(k) => { return Some(k) }
      &Remap::Alias(k) => { i = k; }
    }
  }
}

struct Ctx {
  todo: Vec<usize>,
  seen: HashSet<usize>,
  prev: HashMap<usize, usize>,
  next: HashMap<usize, usize>,
}

impl Ctx {
  fn reset(&mut self) {
    self.todo.clear();
    self.seen.clear();
    self.prev.clear();
    self.next.clear();
  }
}

fn mark(ctx: &mut Ctx, a: usize, b: usize, is_jump: bool) {
  if ctx.seen.contains(&b) {
    if let Some(c) = ctx.prev.remove(&b) {
      ctx.next.remove(&c);
    }
  } else {
    ctx.seen.insert(b);
    ctx.todo.push(b);
    if is_jump {
      ctx.next.insert(a, b);
      ctx.prev.insert(b, a);
    }
  }
}

pub fn simplify(module: &Module) -> Module {
  let Module { funs, code } = module;
  let mut o_code = Vec::new();
  let mut o_funs = Vec::new();
  let mut remap = vec![Remap::None; code.len()];
  let mut ctx =
    Ctx {
      todo: Vec::new(),
      seen: HashSet::new(),
      prev: HashMap::new(),
      next: HashMap::new(),
    };
  for f in funs.iter() {
    let o_f_pos = o_code.len();
    ctx.seen.insert(f.pos);
    ctx.todo.push(f.pos);
    while let Some(i) = ctx.todo.pop() {
      for (_, inst) in iter_block_insts(&code, i) {
        match inst {
          &Inst::Jump(a) => {
            mark(&mut ctx, i, a, true);
          }
          &Inst::Cond(_, a, b) => {
            mark(&mut ctx, i, a, false);
            mark(&mut ctx, i, b, false);
          }
          &Inst::Call(_, a) => {
            mark(&mut ctx, i, a, false);
          }
          _ => {
          }
        }
      }
    }
    let mut seen_: Vec<_> = ctx.seen.iter().cloned().collect();
    seen_.sort();
    let mut arg = Vec::new();
    let mut out = Vec::new();
    for &i in seen_.iter() {
      if ! ctx.prev.contains_key(&i) {
        let mut i = i;
        let mut has_prev = false;
        loop {
          let has_next = ctx.next.contains_key(&i);
          if has_next {
            let Inst::Label(_, n) = code[i] else { unreachable!() };
            out.extend((0 .. n).map(|_| usize::MAX));
          }
          for (j, &inst) in iter_block_insts(&code, i) {
            match inst {
              Inst::Pop(k) if has_prev => {
                remap[j] = Remap::Alias(arg[k]);
              }
              Inst::Put(k, x) if has_next => {
                out[k] = x;
              }
              Inst::Jump(..) if has_next => {
              }
              Inst::Label(..) if has_prev => {
              }
              _ => {
                remap[j] = Remap::Value(o_code.emit(inst));
              }
            }
          }
          arg.clear();
          if ! has_next { break }
          i = ctx.next[&i];
          has_prev = true;
          std::mem::swap(&mut arg, &mut out);
        }
      }
    }
    ctx.reset();
    o_funs.push(
      FunDef {
        name: f.name,
        pos: o_f_pos,
        len: o_code.len() - o_f_pos,
      }
    );
  }
  let mut resolved = Vec::new();
  for i in 0 .. code.len() {
    resolved.push(resolve(&remap, i).unwrap_or(usize::MAX));
  }
  let remap = resolved;
  for i in 0 .. o_code.len() {
    o_code[i] = o_code[i].remap(|i| remap[i]);
  }
  Module {
    funs: o_funs.into_boxed_slice(),
    code: o_code.into_boxed_slice(),
  }
}
