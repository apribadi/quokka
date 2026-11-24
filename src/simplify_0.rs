//!
//!
//!

// #![allow(warnings)]

use crate::prelude::*;
use crate::ssa_0::Inst;
use crate::ssa_0::FunDef;
use crate::ssa_0::Module;
use crate::ssa_0::iter_block_insts;

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
  let mut remap = vec![usize::MAX; code.len()];
  let mut o_code = Vec::new();
  let mut o_funs = Vec::new();
  let mut ctx =
    Ctx {
      todo: Vec::new(),
      seen: HashSet::new(),
      prev: HashMap::new(),
      next: HashMap::new(),
    };
  for f in funs.iter() {
    ctx.seen.insert(f.pos);
    ctx.todo.push(f.pos);
    while let Some(i) = ctx.todo.pop() {
      for (_, inst) in iter_block_insts(&code, i) {
        match inst {
          &Inst::Jump(a) => {
            mark(&mut ctx, i, a, true);
          }
          &Inst::Cond(_, a, b) => {
            mark(&mut ctx, i, a, true);
            mark(&mut ctx, i, b, true);
          }
          &Inst::Call(_, a) => {
            mark(&mut ctx, i, a, true);
          }
          _ => {
          }
        }
      }
    }
    let mut seen_: Vec<_> = ctx.seen.iter().cloned().collect();
    seen_.sort();
    for &i in seen_.iter() {
      for (i, &inst) in iter_block_insts(&code, i) {
        remap[i] = o_code.emit(inst);
      }
    }
    ctx.reset();
    let o_f_pos = remap[f.pos];
    let o_f_end = o_code.len();
    let o_f_len = o_f_end - o_f_pos;
    for i in o_f_pos .. o_f_end {
      o_code[i] = o_code[i].remap(|i| remap[i]);
    }
    o_funs.push(
      FunDef {
        name: f.name,
        pos: o_f_pos,
        len: o_f_len,
      }
    );
  }
  Module {
    funs: o_funs.into_boxed_slice(),
    code: o_code.into_boxed_slice(),
  }
}
