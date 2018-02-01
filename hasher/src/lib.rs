extern crate rand;
extern crate ring;
extern crate spc;

mod time;

use rand::Rng;
use rand::thread_rng;
use ring::digest::SHA256;
use ring::digest::digest;
use spc::sbox;
use time::nsnow;

const COMPUTE_WIDTH: usize = 32;
const DATA_BYTES: usize = 1 * 1_024 * 1024;

#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	*sbox() = nsnow().unwrap();

	let data: Vec<u8> = (0..DATA_BYTES).map(|_| thread_rng().gen()).collect();
	let data = data.into_boxed_slice();

	let count = DATA_BYTES / COMPUTE_WIDTH;
	let sum: i64 = (0..count).map(|num| {
		let lo = num * 16;
		let ts = nsnow().unwrap();
		digest(&SHA256, &data[lo..lo + COMPUTE_WIDTH]);
		nsnow().unwrap() - ts
	}).sum();

	println!("{}", sum as usize / count);
}
