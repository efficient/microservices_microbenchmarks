#[cfg(not(feature = "no_mangle_main"))]
extern crate ipc;
#[cfg(feature = "no_mangle_main")]
extern crate spc;

mod time;

#[cfg(not(feature = "no_mangle_main"))]
use ipc::SMem;
#[cfg(feature = "no_mangle_main")]
use spc::sbox;
use time::nsnow;

#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	*sbox() = nsnow().unwrap();
}

#[cfg(not(feature = "no_mangle_main"))]
fn sbox<'a>() -> SMem<'a, i64> {
	use std::env::args;

	let s = args().skip(1).next().unwrap();
	let u = s.parse().unwrap();

	SMem::from(u).unwrap()
}
