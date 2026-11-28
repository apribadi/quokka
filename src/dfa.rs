
#[inline(never)]
pub fn next(x: u64, t: &[[u64; 1]; 256], b: [u8; 5]) -> u64 {
  let mut x = x;
  for &b in b.iter() {
    let [u, ..] = t[b as usize];
    x = u >> (x & 0x3f);
  }
  x
}

#[derive(Clone, Copy)]
pub struct X(pub u64);

#[derive(Clone, Copy)]
pub struct Y(pub u64);

#[derive(Clone, Copy)]
pub struct F(pub u64, pub u64);
