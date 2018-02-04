use std::cell::Cell;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Error;
use std::mem::transmute;
use std::ops::Deref;
use std::ops::DerefMut;
use std::os::raw::c_char;
use std::os::raw::c_long;
use std::os::raw::c_void;
use std::ptr::null;
use std::ptr::null_mut;
use std::panic::catch_unwind;
use std::slice;

#[cfg(feature = "preserve_loaded")]
const PRESERVE_LOADED_LIBS: bool = true;
#[cfg(not(feature = "preserve_loaded"))]
const PRESERVE_LOADED_LIBS: bool = false;

thread_local! {
	static PREEMPTIBLE: Cell<bool> = Cell::new(false);
}

pub fn setup_preemption(quantum_us: i64, limit_ns: i64, start_time: &i64) -> Result<(), String> {
	if unsafe {
		preempt_setup(quantum_us, limit_ns, PREEMPTIBLE.with(|preemptible| preemptible.as_ptr()), start_time, unwind as *const c_void)
	} {
		Ok(())
	} else {
		Err(format!("{}", Error::last_os_error()))
	}
}

pub fn query_preemption() -> Option<&'static [i64]> {
	let ptr = unsafe {
		preempt_recent_ns()
	};

	if ! ptr.is_null() {
		Some(unsafe {
			slice::from_raw_parts(preempt_recent_ns(), 1 << 8)
		})
	} else {
		None
	}
}

pub struct LibFun {
	lib: *const c_void,
	fun: Box<FnMut(i64) -> Option<i64>>,
}

impl LibFun {
	pub fn new(sofile: &CStr) -> Result<Self, String> {
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
				fun: Box::new(move |sboxed| {
					*sbox = sboxed;

					PREEMPTIBLE.with(|preemptible| {
						preemptible.set(true);
						let success = catch_unwind(fun);
						preemptible.set(false);

						success
					}).ok()?;

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

	pub fn new_from_ptr(libname: *const c_char) -> Result<Self, String> {
		Self::new(unsafe {
			CStr::from_ptr(libname)
		})
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
	type Target = FnMut(i64) -> Option<i64>;

	fn deref(&self) -> &Self::Target {
		&*self.fun
	}
}

impl DerefMut for LibFun {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.fun
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
	fn preempt_setup(quantum_us: c_long, limit_ns: c_long, enforcing: *const bool, checkpoint: *const c_long, punishment: *const c_void) -> bool;
	fn preempt_recent_ns() -> *const c_long;

	fn dl_load(exec: *mut LibFunny, sofile: *const c_char, preserve: bool) -> *const c_char;
	fn dl_unload(exec: LibFunny);
}

extern "C" fn unwind() {
	panic!();
}
