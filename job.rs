use std::cell::Cell;
use std::env::Args;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;

const OBJS_PER_DIR: usize  = 10_000;
const WARMUP_TRIALS: usize =      0; // 0 means the number of distinct object files

#[derive(Clone)]
pub struct FixedCString ([u8; 48]);

impl FixedCString {
	pub fn new() -> Self {
		FixedCString ([0; 48])
	}

	pub fn from(content: &str) -> Self {
		let mut container = Self::new();

		let content = content.as_bytes();
		assert!(content.len() + 1 < container.len());

		for index in 0..content.len() {
			container[index] = content[index];
		}
		container[content.len()] = b'\0';

		container
	}
}

impl Deref for FixedCString {
	type Target = [u8];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for FixedCString {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Eq for FixedCString {}

impl Hash for FixedCString {
	fn hash<T: Hasher>(&self, hasher: &mut T) {
		(**self).hash(hasher)
	}
}

impl PartialEq for FixedCString {
	fn eq(&self, other: &Self) -> bool {
		**self == **other
	}
}

thread_local! {
	static WARMUP: Cell<usize> = Cell::new(WARMUP_TRIALS);
}

#[derive(Clone)]
pub struct Job<T> {
	pub uservice_path: T,
	pub invocation_latency: i64,
	pub completion_time: i64,
}

impl<T> Job<T> {
	pub fn new(path: T) -> Self {
		Job {
			uservice_path: path,
			invocation_latency: 0,
			completion_time: 0,
		}
	}
}

impl<T> Display for Job<T> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		write!(f, "{} {}", self.invocation_latency as f64 / 1_000.0, self.completion_time as f64 / 1_000.0)
	}
}

pub fn joblist<T: Clone, F: Fn(&str) -> T>(svcnames: F, numobjs: usize, numjobs: usize) -> Box<[Job<T>]> {
	let oneshot = |_| Job::new(svcnames(""));
	let multishot = |index| Job::new(svcnames(&format!("{}/{}", index / OBJS_PER_DIR, index % OBJS_PER_DIR)));
	let fun: &Fn(_) -> Job<T> = match numobjs {
		1 => &oneshot,
		_ => &multishot,
	};

	let warmup = WARMUP.with(|warmup| {
		let res = warmup.get();
		if res == 0 {
			warmup.set(numobjs);
			numobjs
		} else {
			res
		}
	});

	let jobs: Vec<_> = (0..numobjs).map(fun).cycle().take(numjobs + warmup).collect();

	jobs.into_boxed_slice()
}

pub fn printstats<T: Clone>(jobs: &[Job<T>]) {
	let warmup = WARMUP.with(|warmup| {
		warmup.get()
	});
	for job in jobs.iter().skip(warmup) {
		println!("{}", job);
	}
}

pub fn args(extra_usage: &str) -> Result<(String, usize, usize, bool, Args), (i32, String)> {
	use std::env::args;

	let mut args = args();
	let prog = args.next().unwrap_or(String::from("<program>"));
	let usage = format!("USAGE: {} [-s] <svcname> [<numfuns> [numtrials{}{}]]", prog, if extra_usage.is_empty() { "" } else { " " }, extra_usage);

	let streams = args.next().ok_or((1, usage.clone()))?;
	let svcname;
	let streams = if streams == "-s" {
		svcname = args.next().ok_or((1, usage.clone()))?;
		true
	} else {
		svcname = streams;
		false
	};

	Ok(if let Some(numobjs) = args.next() {
		(
			svcname,
			numobjs.parse().or((Err((2, String::from("<numfuns>, if provided, must be a nonnegative integer")))))?,
			args.next().unwrap_or(numobjs).parse().or(Err((2, String::from("[numtrials], if provided, must be a nonnegative integer"))))?,
			streams,
			args,
		)
	} else {
		(svcname, 1, 1, streams, args)
	})
}
