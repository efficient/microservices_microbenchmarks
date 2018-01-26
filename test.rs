#[allow(dead_code)]
mod ipc;
mod time;

use ipc::SMem;
use std::env::args;
use time::nsnow;

#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	let mut sbox: SMem<i64> = SMem::from(args().skip(1).next().unwrap().parse().unwrap()).unwrap();

	*sbox = nsnow().unwrap();
}
