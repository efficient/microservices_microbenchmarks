extern crate rust_utils;

use std::mem::forget;

#[allow(unreachable_code)]
fn main() {
	let alloc = Box::new(false);
	println!("{}", alloc);
	forget(alloc);
}
