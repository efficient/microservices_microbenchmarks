use std::ffi::CStr;
use std::ffi::CString;
use std::mem::forget;
use std::ops::Deref;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ptr::null_mut;

#[repr(C)]
pub struct LibFun {
	lib: *const c_void,
	fun: fn(),
}

impl LibFun {
	pub fn new(libname: &str) -> Result<Self, String> {
		let mut exec = LibFun {
			lib: null_mut(),
			fun: ether,
		};
		let sofile = format!("./lib{}.so", libname);
		let sofile = CString::new(&*sofile).map_err(|or| format!("{}", or))?;
		let errmsg = unsafe {
			dl_load(&mut exec, sofile.as_ptr())
		};

		if errmsg.is_null() {
			Ok(exec)
		} else {
			forget(exec);

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
			dl_unload(self);
		}
	}
}

impl Deref for LibFun {
	type Target = fn();

	fn deref(&self) -> &Self::Target {
		&self.fun
	}
}

#[allow(improper_ctypes)]
#[link(name = "runtime")]
extern "C" {
	fn dl_load(exec: *mut LibFun, sofile: *const c_char) -> *const c_char;
	fn dl_unload(exec: *mut LibFun);
}

fn ether() {
	unreachable!();
}
