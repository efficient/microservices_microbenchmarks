#[no_mangle]
pub fn main() {
	let alloc = Box::new(false);
	println!("{}", alloc);
	panic!();
}
