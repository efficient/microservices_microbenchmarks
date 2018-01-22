#[no_mangle]
pub fn call(fun: extern "C" fn()) -> bool {
	use std::panic::catch_unwind;
	use std::process::exit;

	let success = catch_unwind(|| fun()).is_ok();
	// This will call std::sys_common::cleanup for us.
	exit(success as i32);
}

#[no_mangle]
pub fn unwind() -> ! {
	use std::panic::resume_unwind;

	resume_unwind(Box::new(()));
}
