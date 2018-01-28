use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;

pub struct RingBuffer<T> (Box<[T]>);

impl<T> RingBuffer<T> {
	pub fn new(buffer: Box<[T]>) -> Self {
		RingBuffer (buffer)
	}
}

impl<T> Deref for RingBuffer<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

impl<T> DerefMut for RingBuffer<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.0
	}
}

impl<T> Index<usize> for RingBuffer<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index % self.0.len()]
	}
}

impl<T> IndexMut<usize> for RingBuffer<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		let len = self.0.len();

		&mut self.0[index % len]
	}
}
