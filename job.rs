const NUM_USERVICES: usize = 10000;

pub struct Job<T> {
	pub uservice_path: T,
	pub invocation_latency: i64,
}

pub fn joblist<T, F: FnMut(&str) -> T>(svcnames: &mut F, numjobs: usize) -> Box<[Job<T>]> {
	match numjobs {
		1 => Box::new([Job {
			uservice_path: svcnames(""),
			invocation_latency: 0,
		}]),
		_ => {
			let list: Vec<_> = (0..numjobs).map(|index| Job {
				uservice_path: svcnames(&format!("{}", index % NUM_USERVICES)),
				invocation_latency: 0,
			}).collect();

			list.into_boxed_slice()
		},
	}
}

pub fn args() -> Result<(String, usize), (i32, String)> {
	use std::env::args;

	let mut args = args();
	let prog = args.next().unwrap_or(String::from("<program>"));
	let usage = format!("USAGE: {} <svcname> [numjobs]", prog);

	Ok((
		args.next().ok_or((1, usage))?,
		args.next().unwrap_or(String::from("1")).parse().or(Err((2, String::from("[numjobs], if provided, must be a nonnegative integer"))))?,
	))
}
