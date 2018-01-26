use std::ffi::CStr;
use std::ffi::CString;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ptr::null;
use std::panic::catch_unwind;

pub struct LibFun {
	lib: *const c_void,
	fun: Box<Fn() -> bool>,
}

impl LibFun {
	pub fn new(libname: &str) -> Result<Self, String> {
		let mut exec = LibFunny {
			lib: null(),
			fun: null(),
		};
		let sofile = format!("./lib{}.so", libname);
		let sofile = CString::new(&*sofile).map_err(|or| format!("{}", or))?;
		let errmsg = unsafe {
			dl_load(&mut exec, sofile.as_ptr())
		};

		if errmsg.is_null() {
			debug_assert!(! exec.lib.is_null());
			debug_assert!(! exec.fun.is_null());

			let fun: fn() = unsafe {
				transmute(exec.fun)
			};

			Ok(LibFun {
				lib: exec.lib,
				fun: Box::new(move || catch_unwind(fun).is_ok()),
			})
		} else {
			let msg = unsafe {
				CStr::from_ptr(errmsg)
			};
			let msg = msg.to_str().map_err(|or| format!("{}", or))?;

			Err(String::from(msg))
		}
	}
}

impl Drop for LibFun {
	fn drop(&mut self) {
		unsafe {
			dl_unload(LibFunny {
				lib: self.lib,
				fun: null(),
			});
		}
	}
}

impl Deref for LibFun {
	type Target = Fn() -> bool;

	fn deref(&self) -> &Self::Target {
		&*self.fun
	}
}

#[repr(C)]
struct LibFunny {
	lib: *const c_void,
	fun: *const c_void,
}

#[link(name = "runtime")]
extern "C" {
	fn dl_load(exec: *mut LibFunny, sofile: *const c_char) -> *const c_char;
	fn dl_unload(exec: LibFunny);
}
