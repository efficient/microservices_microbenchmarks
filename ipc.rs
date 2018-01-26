use std::io::Error;
use std::io::Result;
use std::mem::size_of;
use std::ops::Deref;
use std::ops::DerefMut;
use std::os::raw::c_int;
use std::os::raw::c_ulong;
use std::os::raw::c_void;
use std::ptr::null_mut;

const PERMS: c_int = 0o600;

pub struct SMem<'a, T: 'a> {
	id: u32,
	data: &'a mut T,
}

impl<'a, T> SMem<'a, T> {
	pub fn new(val: T) -> Result<Self> {
		let mut region = SMemInternal {
			id: 0,
			data: null_mut(),
		};

		if unsafe { salloc(&mut region, size_of::<T>() as c_ulong, PERMS) } {
			let mut region = smem_external(region).unwrap();

			*region = val;

			Ok(region)
		} else {
			Err(Error::last_os_error())
		}
	}

	pub fn from(id: u32) -> Result<Self> {
		let mut region = SMemInternal {
			id: id as i32,
			data: null_mut(),
		};

		if unsafe { sretrieve(&mut region) } {
			Ok(smem_external(region).unwrap())
		} else {
			Err(Error::last_os_error())
		}
	}

	pub fn id(&self) -> u32 {
		self.id
	}
}

impl<'a, T> Drop for SMem<'a, T> {
	fn drop(&mut self) {
		unsafe {
			sdealloc(SMemInternal {
				id: self.id as i32,
				data: self.data as *mut T as *mut c_void,
			})
		}
	}
}

impl<'a, T> Deref for SMem<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.data
	}
}

impl<'a, T> DerefMut for SMem<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.data
	}
}

fn smem_external<'a, T>(internal: SMemInternal) -> Option<SMem<'a, T>> {
	debug_assert!(internal.id >= 0);

	let rfc = internal.data as *mut T;
	let rfc = unsafe {
		rfc.as_mut()
	}?;

	Some(SMem {
		id: internal.id as u32,
		data: rfc,
	})
}

#[repr(C)]
pub struct SMemInternal {
	id: c_int,
	data: *mut c_void,
}

#[link(name = "ipc")]
extern "C" {
	fn sretrieve(region: *mut SMemInternal) -> bool;
	fn salloc(region: *mut SMemInternal, size: c_ulong, flags: c_int) -> bool;
	fn sdealloc(region: SMemInternal);
}
