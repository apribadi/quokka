//!
//!
//!

// #![allow(warnings)]

use crate::prelude::*;
use crate::ssa_0::Inst;
use crate::ssa_0::FunDef;
use crate::ssa_0::Module;
use crate::ssa_0::iter_block_insts;

/*
enum Mult<A> {
  None,
  Many,
  Uniq(A),
}

struct Info {
  prev: Mult<usize>,
  next: Mult<usize>,
}
*/

pub fn simplify(module: &Module) -> Module {
  module.clone()
  /*
  let Module { funs, code } = module;
  let mut remap = vec![usize::MAX; code.len()];
  let mut todo = Vec::new();
  let mut seen = HashMap::new();
  let mut o_code = Vec::new();
  let mut o_funs = Vec::new();
  for f in funs.iter() {
    seen.insert(f.pos, Info { prev: Mult::None, next: Mult::None });
    todo.push(f.pos);
    while let Some(i) = todo.pop() {
      for (_, inst) in iter_block_insts(&code, i) {
        if let Inst::Goto(a) = *inst {
          let info = seen.get_mut(&i).unwrap();
          match info.next {
            _ => ()
          }
          match seen.entry(a) {
            Vacant(e) => {
              e.insert(Info { prev: Mult::Uniq(i), next: Mult::None });
              todo.push(a);
            }
            Occupied(e) => {
              let info = e.into_mut();
              match info.prev {
                Mult::None => panic!(),
                Mult::Many => (),
                Mult::Uniq(_) => info.prev = Mult::Many,
              }
            }
          }
        }
      }
    }
    let mut seen_: Vec<_> = seen.keys().cloned().collect();
    seen_.sort();
    for &i in seen_.iter() {
      for (i, &inst) in iter_block_insts(&code, i) {
        remap[i] = o_code.emit(inst);
      }
    }
    seen.clear();
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
  */
}
