pub(crate) use hashbrown::hash_map::HashMap;

pub(crate) fn concat<T>(x: Box<[T]>, y: Box<[T]>) {
  let mut a = Vec::<T>::from(x);
  a.extend(y);
  a.into_boxed_slice();
}

pub(crate) trait PopSeq<T> {
  fn pop_seq(&mut self, _: usize) -> impl Iterator<Item = T> + '_;
}

impl<T> PopSeq<T> for Vec<T> {
  fn pop_seq(&mut self, n: usize) -> impl Iterator<Item = T> + '_ {
    let m = self.len();
    self.drain(m - n .. m)
  }
}

pub(crate) trait PopOne<T> {
  fn pop_one(&mut self) -> T;
}

impl<T> PopOne<T> for Vec<T> {
  fn pop_one(&mut self) -> T {
    self.pop().unwrap()
  }
}

