use std::cell::RefCell;
use std::io::Error;
use std::io::Result;
use std::mem::replace;
use std::os::raw::c_int;
use std::panic::set_hook;

thread_local! {
	static HOOK: RefCell<Box<Fn()>> = RefCell::new(Box::new(|| ()));
}

pub fn exit(code: i32) -> ! {
	use std::process::exit;

	HOOK.with(|hook| hook.borrow()());
	exit(code);
}

pub fn kill(id: i32) -> Result<()> {
	const SIGKILL: i32 = 9;

	extern "C" {
		fn kill(pid: c_int, sig: c_int) -> i32;
	}

	if unsafe {
		kill(id, SIGKILL)
	} != 0 {
		Err(Error::last_os_error())?
	}

	Ok(())
}

pub fn kill_at_exit(id: i32) {
	HOOK.with(|hook| {
		replace(&mut *hook.borrow_mut(), Box::new(move || kill(id).unwrap_or_else(|err| eprintln!("Failed to kill all child processes: {}", err))));
	});

	set_hook(Box::new(|_| HOOK.with(|hook| hook.borrow_mut()())));
}
