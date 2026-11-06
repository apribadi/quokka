#![allow(unused)]

pub(crate) use hashbrown::hash_map::HashMap;
pub(crate) use hashbrown::hash_map::Entry;
pub(crate) use hashbrown::hash_set::HashSet;
pub(crate) use std::fmt::Write;
pub(crate) use hashbrown::hash_map::Entry::*;
pub(crate) use std::num::NonZeroU64;
pub(crate) use std::debug_assert_matches;

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

pub(crate) trait Emit<T> {
  fn emit(&mut self, _: T) -> usize;
}

impl<T> Emit<T> for Vec<T> {
  fn emit(&mut self, value: T) -> usize {
    let i = self.len();
    self.push(value);
    i
  }
}

