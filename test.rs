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
use std::process::exit;
use time::nsnow;

enum Argument<T> {
	MutableReference((T, Option<i32>)),
	#[cfg_attr(feature = "no_mangle_main", allow(dead_code))]
	UnparseableString(String),
}

#[cfg_attr(feature = "no_mangle_main", allow(unused_mut))]
#[cfg_attr(feature = "no_mangle_main", no_mangle)]
pub fn main() {
	match sbox() {
		Argument::MutableReference((mut loc, status)) => {
			*loc = nsnow().unwrap();

			if let Some(status) = status {
				exit(status);
			}
		},
		Argument::UnparseableString(addrs) => {
			let mut addrs = addrs.split_whitespace();
			let them = addrs.next().unwrap();
			let me = addrs.next().unwrap();
			let index = addrs.next().unwrap();
			let mut index: usize = index.parse().unwrap();

			let socket = UdpSocket::bind(&me).unwrap();
			socket.send_to(index.bytes(), &them).unwrap();

			loop {
				let mut index = 0usize;
				socket.recv(index.bytes()).unwrap();

				let mut mess = (index, nsnow().unwrap());
				socket.send_to(mess.bytes(), &them).unwrap();
			}
		},
	}
}

#[cfg(not(feature = "no_mangle_main"))]
fn sbox<'a>() -> Argument<SMem<'a, i64>> {
	use std::env::args;

	let mut args = args();
	let s = args.by_ref().skip(1).next().unwrap();

	if let Ok(u) = s.parse() {
		Argument::MutableReference((SMem::from(u).unwrap(), args.next().map(|it| it.parse().unwrap())))
	} else {
		Argument::UnparseableString(s)
	}
}

#[cfg(feature = "no_mangle_main")]
fn sbox() -> Argument<&'static mut i64> {
	use spc::sbox;

	Argument::MutableReference((sbox(), None))
}
