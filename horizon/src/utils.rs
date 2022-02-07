//! useful utilities and things
use std::{iter, slice};

/// ring buffer thing
///
/// todo: use a vecdeque instead
#[derive(Clone, Debug)]
pub struct RingBuffer<T: Clone>(Vec<T>, usize);
impl<T: Clone> RingBuffer<T> {
	pub fn new_fill(size: usize, value: T) -> Self {
		Self(vec![value; size], 0)
	}
	pub fn push(&mut self, item: T) {
		self.0[self.1] = item;
		self.1 += 1;
		if self.1 == self.0.len() { self.1 = 0; }
	}
	pub fn iter(&self) -> iter::Chain<slice::Iter<'_, T>, slice::Iter<'_, T>> {
		let (l, r) = self.0.split_at(self.1);
		r.iter().chain(l.iter())
	}
	pub fn len(&self) -> usize {
		self.0.len()
	}
}
