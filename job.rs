use std::cell::Cell;
use std::env::Args;

const OBJS_PER_DIR: usize  = 10_000;
const WARMUP_TRIALS: usize =      0; // 0 means the number of distinct object files

pub type FixedCString = [u8; 24];

pub fn fixed_c_string() -> FixedCString {
	[0; 24]
}

pub fn as_fixed_c_string(content: &str) -> FixedCString {
	let mut container = fixed_c_string();

	let content = content.as_bytes();
	assert!(content.len() + 1 < container.len());

	for index in 0..content.len() {
		container[index] = content[index];
	}
	container[content.len()] = b'\0';

	container
}

thread_local! {
	static WARMUP: Cell<usize> = Cell::new(WARMUP_TRIALS);
}

#[derive(Clone)]
pub struct Job<T> {
	pub uservice_path: T,
	pub invocation_latency: i64,
}

pub fn joblist<T: Clone, F: Fn(&str) -> T>(svcnames: &mut F, numobjs: usize, numjobs: usize) -> Box<[Job<T>]> {
	let oneshot = |_| Job {
		uservice_path: svcnames(""),
		invocation_latency: 0,
	};
	let multishot = |index| Job {
		uservice_path: svcnames(&format!("{}/{}", index / OBJS_PER_DIR, index % OBJS_PER_DIR)),
		invocation_latency: 0,
	};
	let fun: &Fn(_) -> Job<T> = match numobjs {
		1 => &oneshot,
		_ => &multishot,
	};

	let warmup = WARMUP.with(|warmup| {
		let mut res = warmup.take();
		if res == 0 {
			res = numobjs;
		}
		warmup.set(res);
		res
	});

	let jobs: Vec<_> = (0..numobjs).map(fun).cycle().take(numjobs + warmup).collect();

	jobs.into_boxed_slice()
}

pub fn printstats<T: Clone>(jobs: &Box<[Job<T>]>) {
	let warmup = WARMUP.with(|warmup| {
		let res = warmup.take();
		warmup.set(res);
		res
	});
	for job in jobs.iter().skip(warmup) {
		println!("{}", job.invocation_latency as f64 / 1_000.0);
	}
}

pub fn args(extra_usage: &str) -> Result<(String, usize, usize, Args), (i32, String)> {
	use std::env::args;

	let mut args = args();
	let prog = args.next().unwrap_or(String::from("<program>"));
	let usage = format!("USAGE: {} <svcname> [<numfuns> <numtrials>{}{}]", prog, if extra_usage.is_empty() { "" } else { " " }, extra_usage);

	let svcname = args.next().ok_or((1, usage.clone()))?;

	Ok(if let Some(numobjs) = args.next() {
		(
			svcname,
			numobjs.parse().or((Err((2, String::from("<numfuns>, if provided, must be a nonnegative integer")))))?,
			args.next().unwrap_or(usage.clone()).parse().or(Err((2, String::from("<numtrials>, if provided, must be a nonnegative integer"))))?,
			args,
		)
	} else {
		(svcname, 1, 1, args)
	})
}
