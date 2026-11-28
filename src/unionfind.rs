use crate::prelude::*;
use std::cell::Cell;
use std::hint::unreachable_unchecked;
use std::ops::Index;
use std::ops::IndexMut;

pub struct UnionFind<T>(Vec<Node<T>>);

enum Node<T> {
  Link(Cell<usize>),
  Root(T),
}

impl<T> Node<T> {
  unsafe fn root_unchecked(self) -> T {
    let Node::Root(value) = self else { unsafe { unreachable_unchecked() } };
    value
  }

  unsafe fn root_unchecked_ref(&self) -> &T {
    let Node::Root(value) = self else { unsafe { unreachable_unchecked() } };
    value
  }

  unsafe fn root_unchecked_mut(&mut self) -> &mut T {
    let Node::Root(value) = self else { unsafe { unreachable_unchecked() } };
    value
  }
}

impl<T> UnionFind<T> {
  pub fn new() -> Self {
    Self(Vec::new())
  }

  pub fn push(&mut self, value: T) -> usize {
    let n = self.0.len();
    self.0.push(Node::Root(value));
    n
  }

  unsafe fn find_unchecked(&self, index: usize) -> usize {
    debug_assert!(index < self.0.len());

    // path splitting

    if let &Node::Link(ref a) = unsafe { self.0.get_unchecked(index) } {
      let mut i = a.get();
      let mut a = a;

      while let &Node::Link(ref b) = unsafe { self.0.get_unchecked(i) } {
        i = b.get();
        a.set(i);
        a = b;
      }

      return i;
    }

    return index;
  }

  /// Returns a mutable reference to the value for `index`'s equivalence class.
  /// If `other` is in a different equivalence class, combines the two
  /// eqivalence classes and removes and returns the old value for `other`'s
  /// equivalence class.

  pub fn union(&mut self, index: usize, other: usize) -> (&mut T, Option<T>) {
    let n = self.0.len();

    assert!(index < n && other < n);

    let i = unsafe { self.find_unchecked(index) };
    let j = unsafe { self.find_unchecked(other) };

    // index by rank - all links point to lower indices
    //
    // If indices are assigned randomly (which is not guaranteed) then there
    // are tighter complexity bounds.

    if i == j {
      return (unsafe { self.get_root_unchecked_mut(i) }, None);
    } else if i < j {
      let a = replace(unsafe { self.0.get_unchecked_mut(j) }, Node::Link(Cell::new(i)));
      return unsafe { (self.get_root_unchecked_mut(i), Some(a.root_unchecked())) };
    } else {
      let a = replace(unsafe { self.0.get_unchecked_mut(i) }, Node::Link(Cell::new(j)));
      let a = replace(unsafe { self.0.get_unchecked_mut(j) }, a);
      return unsafe { (self.get_root_unchecked_mut(j), Some(a.root_unchecked())) };
    }
  }

  pub fn is_equivalent(&self, index: usize, other: usize) -> bool {
    let n = self.0.len();

    assert!(index < n && other < n);

    let i = unsafe { self.find_unchecked(index) };
    let j = unsafe { self.find_unchecked(other) };

    return i == j;
  }

  unsafe fn get_root_unchecked(&self, index: usize) -> &T {
    debug_assert!(index < self.0.len());

    unsafe { self.0.get_unchecked(index).root_unchecked_ref() }
  }

  unsafe fn get_root_unchecked_mut(&mut self, index: usize) -> &mut T {
    debug_assert!(index < self.0.len());

    unsafe { self.0.get_unchecked_mut(index).root_unchecked_mut() }
  }
}

impl<T> Index<usize> for UnionFind<T> {
  type Output = T;

  fn index(&self, index: usize) -> &T {
    assert!(index < self.0.len());

    unsafe { self.get_root_unchecked(self.find_unchecked(index)) }
  }
}

impl<T> IndexMut<usize> for UnionFind<T> {
  fn index_mut(&mut self, index: usize) -> &mut T {
    assert!(index < self.0.len());

    unsafe { self.get_root_unchecked_mut(self.find_unchecked(index)) }
  }
}

impl<T: std::fmt::Display> std::fmt::Display for UnionFind<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for i in 0 .. self.0.len() {
      match &self.0[i] {
      | &Node::Link(ref a) => write!(f, "{}: => {}\n", i, a.get())?,
      | &Node::Root(ref a) => write!(f, "{}: {}\n", i, a)?,
      }
    }
    Ok(())
  }
}
