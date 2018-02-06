use std::process::exit;

const ADDR: &str = "0.0.0.0";
const BASE: u16 = 1024;

fn thread(index: u16) -> ! {
	use std::net::UdpSocket;

	let socket = UdpSocket::bind((ADDR, BASE + index)).unwrap();

	loop {
		match socket.recv_from(&mut []) {
			Ok((_, addr)) => if let Err(or) = socket.send_to(&[], addr) {
				eprintln!("Thread {} send: {}", index, or);
			},
			Err(or) => eprintln!("Thread {} recv: {}", index, or),
		}
	}
}

fn main() {
	use std::thread::spawn;

	let nthreads = args().unwrap_or_else(|err| {
		println!("{}", err);
		exit(1);
	});

	let threads: Vec<_> = (0..nthreads).map(|index| spawn(move || thread(index))).collect();
	for thread in threads {
		if thread.join().is_err() {
			eprintln!("Unable to join on a thread!");
		}
	}
}

fn args() -> Result<u16, String> {
	const UNPARSEABLE: &str = "<numcores> must be a positive integer";

	use std::env::args;

	let mut args = args();
	let name = args.next().unwrap_or(String::from("<executable>"));
	let quest = args.next().ok_or_else(|| format!("USAGE: {} <numcores>", name))?;
	let quest = quest.parse().or(Err(String::from(UNPARSEABLE)))?;

	if quest != 0 {
		Ok(quest)
	} else {
		Err(String::from(UNPARSEABLE))
	}
}
