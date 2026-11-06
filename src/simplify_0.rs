//!
//!
//!

#![allow(warnings)]

use crate::prelude::*;
use crate::ssa_0 as ssa;
use crate::ssa_0::Inst;
use crate::ssa_0::FunDef;
use crate::ssa_0::Module;

pub fn simplify(module: &Module) -> Module {
  let Module { funs, code } = module;
  let mut remap = vec![usize::MAX; code.len()];
  let mut todo = Vec::new();
  let mut seen = HashMap::new();
  let mut o_code = Vec::new();
  let mut o_funs = Vec::new();
  for f in funs.iter() {
    let f_pos = f.pos;
    let f_end = f.pos + f.len;
    seen.insert(f_pos, 1);
    todo.push(f_pos);
    while let Some(i) = todo.pop() {
      let mut i = i + 1;
      while i < f_end {
        let inst = code[i];
        if let Inst::Label(..) = inst {
          break;
        }
        if let Inst::Goto(a) = inst {
          match seen.entry(a) {
            Vacant(e) => { e.insert(1); todo.push(a); }
            Occupied(e) => { *e.into_mut() += 1; }
          }
        }
        i += 1;
      }
    }
    let mut i = f_pos;
    let mut is_output = false;
    while i < f_end {
      let inst = code[i];
      if let Inst::Label(..) = inst {
        is_output = seen.contains_key(&i);
      }
      if is_output {
        remap[i] = o_code.emit(inst);
      }
      i += 1;
    }
    let o_f_pos = remap[f_pos];
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
