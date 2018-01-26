#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	let alloc = Box::new(false);
	println!("{}", alloc);
	panic!();
}
