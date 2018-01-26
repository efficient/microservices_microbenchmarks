mod runtime;

use runtime::LibFun;
use std::process::exit;

fn main() {
	for _ in 0..2 {
		let fun = LibFun::new("test").unwrap_or_else(|or| {
			eprintln!("{}", or);
			exit(2);
		});
		fun();
	}
}
