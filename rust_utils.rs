#[no_mangle]
pub fn unwind() -> ! {
	use std::panic::resume_unwind;

	resume_unwind(Box::new(()));
}
