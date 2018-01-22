#[no_mangle]
pub fn call(fun: extern "C" fn()) -> bool {
	use std::panic::catch_unwind;
	use std::sys_common::cleanup;

	catch_unwind(|| fun()).is_ok()
}

#[no_mangle]
pub fn unwind() -> ! {
	use std::panic::resume_unwind;

	resume_unwind(Box::new(()));
}
