use crate::prelude::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Symbol(NonZeroU64);

pub struct SymbolTable(HashMap<NonZeroU64, String>);

static HASHER: foldhash::quality::FixedState =
  foldhash::quality::FixedState::with_seed(0);

fn intern(s: &str) -> NonZeroU64 {
  if 1 <= s.len() && s.len() <= 8 {
    if s.as_bytes().iter().all(|&c| c != 0) {
      let mut buf = [0u8; 8];
      buf[.. s.len()].copy_from_slice(s.as_bytes());
      let n = u64::from_le_bytes(buf);
      return NonZeroU64::new(n).unwrap();
    }
  }
  let n = <foldhash::quality::FixedState as std::hash::BuildHasher>::hash_one(&HASHER, s);
  let n = n | 1 << 63;
  let n = NonZeroU64::new(n).unwrap();
  n
}

fn is_inline(n: NonZeroU64) -> bool {
  0 <= n.get() as i64
}

impl std::fmt::Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if is_inline(self.0) {
      let buf = self.0.get().to_le_bytes();
      let mut i = 0;
      while i < 8 && buf[i] != 0 { i += 1; }
      write!(f, "{}", str::from_utf8(&buf[.. i]).unwrap())
    } else {
      write!(f, "Symbol({:#X})", self.0.get())
    }
  }
}

impl SymbolTable {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn intern(&mut self, s: &str) -> Symbol {
    let n = intern(s);
    if ! is_inline(n) {
      match self.0.entry(n) {
        Vacant(entry) => {
          entry.insert(String::from(s));
        }
        Occupied(entry) => {
          if s != entry.get() {
            panic!("symbol hash collision")
          }
        }
      }
    }
    Symbol(n)
  }
}
