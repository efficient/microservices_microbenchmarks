use std::cell::RefCell;
use std::io::Error;
use std::io::Result;
use std::mem::replace;
use std::os::raw::c_int;
use std::panic::PanicInfo;
use std::panic::set_hook;
use std::panic::take_hook;

const SIGKILL: i32 = 9;
const SIGTERM: i32 = 15;
const WNOHANG: i32 = 1;

thread_local! {
	static DEFAULT_HOOK: RefCell<Option<Box<Fn(&PanicInfo)>>> = RefCell::new(None);
	static HOOK: RefCell<Box<Fn()>> = RefCell::new(Box::new(|| ()));
}

pub fn exit(code: i32) -> ! {
	use std::process::exit;

	HOOK.with(|hook| hook.borrow()());
	exit(code);
}

pub fn term(id: i32) -> Result<()> {
	kill(id, SIGTERM)
}

fn kill(id: i32, sig: i32) -> Result<()> {
	extern "C" {
		fn kill(pid: c_int, sig: c_int) -> i32;
	}

	if unsafe {
		kill(id, sig)
	} != 0 {
		Err(Error::last_os_error())?
	}

	Ok(())
}

pub fn kill_at_exit(id: i32) {
	HOOK.with(|hook| {
		replace(&mut *hook.borrow_mut(), Box::new(move || kill(id, SIGKILL).unwrap_or_else(|err| eprintln!("Failed to kill all child processes: {}", err))));
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

pub fn nowait() -> Result<Option<i32>> {
	#[link(name = "pgroup")]
	extern "C" {
		fn waitpid(pid: c_int, status: Option<&mut c_int>, opts: c_int) -> c_int;
		fn wexitstatus(wstatus: Option<&mut c_int>) -> Option<&mut c_int>;
	}

	let mut status = 0;
	let pid = unsafe {
		waitpid(0, Some(&mut status), WNOHANG)
	};

	if pid == -1 {
		Err(Error::last_os_error())?
	}

	Ok(if pid != 0 {
		unsafe {
			wexitstatus(Some(&mut status))
		}
	} else {
		None
	}.map(|status| *status))
}
