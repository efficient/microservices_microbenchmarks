use std::ffi::CStr;
use std::ffi::CString;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_char;
use std::os::raw::c_long;
use std::os::raw::c_void;
use std::ptr::null;
use std::ptr::null_mut;
use std::panic::catch_unwind;

#[cfg(feature = "preserve_loaded")]
const PRESERVE_LOADED_LIBS: bool = true;
#[cfg(feature = "cleanup_loaded")]
const PRESERVE_LOADED_LIBS: bool = false;

pub struct LibFun {
	lib: *const c_void,
	fun: Box<Fn() -> Option<i64>>,
}

impl LibFun {
	pub fn new(sofile: &CString) -> Result<Self, String> {
		let mut exec = LibFunny {
			lib: null(),
			fun: null(),
			sbox: null_mut(),
		};

		let errmsg = unsafe {
			dl_load(&mut exec, sofile.as_ptr(), PRESERVE_LOADED_LIBS)
		};

		if errmsg.is_null() {
			debug_assert!(! exec.lib.is_null());
			debug_assert!(! exec.fun.is_null());

			let fun: fn() = unsafe {
				transmute(exec.fun)
			};
			let sbox = unsafe {
				exec.sbox.as_mut()
			}.expect("Library has no static storage space!");

			Ok(LibFun {
				lib: exec.lib,
				fun: Box::new(move || {
					catch_unwind(fun).ok()?;

					Some(*sbox)
				}),
			})
		} else {
			let msg = unsafe {
				CStr::from_ptr(errmsg)
			};
			let msg = msg.to_str().map_err(|or| format!("{}", or))?;

			Err(String::from(msg))
		}
	}

	pub fn new_from_str(libname: &str) -> Result<Self, String> {
		let sofile = format!("./lib{}.so", libname);
		let sofile = CString::new(&*sofile).map_err(|or| format!("{}", or))?;

		Self::new(&sofile)
	}
}

impl Drop for LibFun {
	fn drop(&mut self) {
		unsafe {
			dl_unload(LibFunny {
				lib: self.lib,
				fun: null(),
				sbox: null_mut(),
			});
		}
	}
}

impl Deref for LibFun {
	type Target = Fn() -> Option<i64>;

	fn deref(&self) -> &Self::Target {
		&*self.fun
	}
}

#[repr(C)]
struct LibFunny {
	lib: *const c_void,
	fun: *const c_void,
	sbox: *mut c_long,
}

#[link(name = "runtime")]
extern "C" {
	fn dl_load(exec: *mut LibFunny, sofile: *const c_char, preserve: bool) -> *const c_char;
	fn dl_unload(exec: LibFunny);
}

#[cfg(not(any(feature = "preserve_loaded", feature = "cleanup_loaded")))]
compile_error!("Must select an *_loaded personality via '--feature' or '--cfg feature='!");
