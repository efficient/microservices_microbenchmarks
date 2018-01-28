extern crate bytes;
#[cfg(not(feature = "no_mangle_main"))]
extern crate ipc;
#[cfg(feature = "no_mangle_main")]
extern crate spc;

mod time;

use bytes::Bytes;
#[cfg(not(feature = "no_mangle_main"))]
use ipc::SMem;
use std::net::UdpSocket;
use time::nsnow;

enum Argument<T> {
	MutableReference(T),
	#[cfg_attr(feature = "no_mangle_main", allow(dead_code))]
	UnparseableString(String),
}

#[cfg_attr(feature = "no_mangle_main", allow(unused_mut))]
#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	match sbox() {
		Argument::MutableReference(mut loc) => *loc = nsnow().unwrap(),
		Argument::UnparseableString(addrs) => {
			let mut addrs = addrs.split_whitespace();
			let them = addrs.next().unwrap();
			let me = addrs.next().unwrap();
			let index = addrs.next().unwrap();
			let mut index: usize = index.parse().unwrap();

			let socket = UdpSocket::bind(&me).unwrap();
			socket.send_to(index.bytes(), &them).unwrap();

			loop {
				socket.recv(&mut []).unwrap();

				let mut ts = nsnow().unwrap();
				socket.send_to(ts.bytes(), &them).unwrap();
			}
		},
	}
}

#[cfg(not(feature = "no_mangle_main"))]
fn sbox<'a>() -> Argument<SMem<'a, i64>> {
	use std::env::args;

	let s = args().skip(1).next().unwrap();

	if let Ok(u) = s.parse() {
		Argument::MutableReference(SMem::from(u).unwrap())
	} else {
		Argument::UnparseableString(s)
	}
}

#[cfg(feature = "no_mangle_main")]
fn sbox() -> Argument<&'static mut i64> {
	use spc::sbox;

	Argument::MutableReference(sbox())
}
