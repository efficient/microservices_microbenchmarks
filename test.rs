//use std::mem::forget;

#[no_mangle]
pub fn main() {
	let alloc = Box::new(false);
	println!("{}", alloc);
	//panic!();
	//forget(alloc);
}
