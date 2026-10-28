#![allow(unused)]

pub(crate) use hashbrown::hash_map::HashMap;
pub(crate) use std::fmt::Write;

pub(crate) fn concat<T>(x: Box<[T]>, y: Box<[T]>) {
  let mut a = Vec::<T>::from(x);
  a.extend(y);
  a.into_boxed_slice();
}

pub(crate) trait PopList<T> {
  fn pop_list(&mut self, _: usize) -> impl Iterator<Item = T> + '_;
}

impl<T> PopList<T> for Vec<T> {
  fn pop_list(&mut self, n: usize) -> impl Iterator<Item = T> + '_ {
    let m = self.len();
    self.drain(m - n .. m)
  }
}

