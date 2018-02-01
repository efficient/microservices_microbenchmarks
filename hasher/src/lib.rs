extern crate rand;
extern crate ring;
extern crate spc;

mod time;

use rand::Rng;
use rand::thread_rng;
use ring::digest::SHA512;
use ring::digest::digest;
use spc::sbox;
use time::nsnow;

const COMPUTE_WIDTH: usize = 64;
const DATA_BYTES: usize = 1_024 * 1_024 * 1024;
const REPORTS_PER_SEC: i64 = 2;

#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	*sbox() = nsnow().unwrap();

	let data: Vec<u8> = (0..DATA_BYTES).map(|_| thread_rng().gen()).collect();
	let data = data.into_boxed_slice();

	let mut ts = nsnow().unwrap();
	let mut count = 0;

	for lo in (0..DATA_BYTES / COMPUTE_WIDTH).map(|num| num * 16) {
		digest(&SHA512, &data[lo..lo + COMPUTE_WIDTH]);

		count += 1;
		if nsnow().unwrap() - ts >= 1_000_000_000 / REPORTS_PER_SEC {
			println!("{}", count * REPORTS_PER_SEC);
			count = 0;
			ts = nsnow().unwrap();
		}
	}
}
