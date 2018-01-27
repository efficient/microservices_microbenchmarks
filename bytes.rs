use std::mem::size_of_val;
use std::slice::from_raw_parts_mut;

pub trait Bytes {
	// Because this trait is intended to provide zero-copy raw access, it would be racy if it
	// accepted non-mut refs.
	fn bytes<'a>(&'a mut self) -> &'a mut [u8];
}

pub trait DefaultBytes {}

impl<T: DefaultBytes> Bytes for T {
	fn bytes<'a>(&'a mut self) -> &'a mut [u8] {
		unsafe {
			from_raw_parts_mut(self as *mut T as *mut u8, size_of_val(self))
		}
	}
}

impl DefaultBytes for () {}

#[test]
fn unit() {
	assert_eq!(0, ().bytes().len());
}

impl Bytes for String {
	fn bytes<'a>(&'a mut self) -> &'a mut [u8] {
		unsafe {
			self.as_bytes_mut()
		}
	}
}

#[test]
fn string() {
	let msg = "The quick brown fox jumps over the lazy hen.";
	assert_eq!(msg.len(), msg.bytes().len());
}
