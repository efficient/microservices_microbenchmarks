extern crate spc;

mod time;

use spc::sbox;
use time::nsnow;

#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	*sbox() = nsnow().unwrap();
}
