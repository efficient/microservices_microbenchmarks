extern crate spc;

mod time;

use spc::sbox;
use time::nsnow;

const DURATION_NS: i64 = 1_000_000_000;

#[no_mangle]
pub fn main() {
	*sbox() = nsnow().unwrap();
	while nsnow().unwrap() - *sbox() < DURATION_NS {}
}
