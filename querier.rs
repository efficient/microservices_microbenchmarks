extern crate spc;

mod time;

use spc::sbox;
use std::cmp::max;
use std::net::UdpSocket;
use time::nsnow;

const ADDR: &str = "192.168.0.1:0";
const BASE: u16 = 1024;
const COMP: i64 = 90_000;
const DEST: &str = "192.168.0.2";
const EXCD: i64 = 110_000;

#[no_mangle]
pub fn main() {
	let ts = nsnow().unwrap();
	let sbox = sbox();
	let port = BASE + (*sbox as u16);

	*sbox = ts;

	let socket = UdpSocket::bind(ADDR).unwrap();
	let (mut sendt, mut sends, mut recvt, mut recvs) = (0, 0, 0, 0);

	let mut cur = nsnow().unwrap();
	let mut sor = cur;
	while cur - ts < COMP {
		socket.send_to(&[], (DEST, port)).unwrap();
		sends += 1;

		sor = nsnow().unwrap();
		sendt += sor - cur;
		if sor - ts >= COMP {
			break;
		}

		socket.recv(&mut []).unwrap();
		recvs += 1;

		cur = nsnow().unwrap();
		recvt += cur - sor;
	}

	if sends != 0 && nsnow().unwrap() >= EXCD {
		*max(&mut sends, &mut recvs) -= 1;
		sendt -= 0.min(sor - cur);
		recvt -= 0.min(cur - sor);
	}

	println!("{} {} {} {}", sends, recvs,
		if sends == 0 { 0.0 } else { sendt as f64 / sends as f64 / 1_000.0 },
		if recvs == 0 { 0.0 } else { recvt as f64 / recvs as f64 / 1_000.0 });
}
