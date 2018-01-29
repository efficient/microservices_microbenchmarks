use std::cell::RefCell;
use std::io::Error;
use std::io::Result;
use std::mem::replace;
use std::os::raw::c_int;
use std::panic::PanicInfo;
use std::panic::set_hook;
use std::panic::take_hook;

thread_local! {
	static DEFAULT_HOOK: RefCell<Option<Box<Fn(&PanicInfo)>>> = RefCell::new(None);
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

	DEFAULT_HOOK.with(|default_hook| {
		if default_hook.borrow().is_none() {
			replace(&mut *default_hook.borrow_mut(), Some(take_hook()));
		}
	});

	set_hook(Box::new(|crash| {
		HOOK.with(|hook| hook.borrow_mut()());
		DEFAULT_HOOK.with(|default_hook| default_hook.borrow().as_ref().unwrap()(crash));
	}));
}

pub fn setpgid(gid: u32) -> Result<u32> {
	extern "C" {
		fn setpgid(pid: c_int, gid: c_int) -> c_int;
	}

	let id = unsafe {
		setpgid(0, gid as c_int)
	};

	if id != 0 {
		Err(Error::last_os_error())?
	}

	Ok(id as u32)
}
