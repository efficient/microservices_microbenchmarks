const WARMUP_TRIALS: usize =  3_000;

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
		uservice_path: svcnames(&format!("{}", index)),
		invocation_latency: 0,
	};
	let fun: &Fn(_) -> Job<T> = match numobjs {
		1 => &oneshot,
		_ => &multishot,
	};

	let jobs: Vec<_> = (0..numobjs).map(fun).cycle().take(numjobs + WARMUP_TRIALS).collect();

	jobs.into_boxed_slice()
}

pub fn printstats<T: Clone>(jobs: &Box<[Job<T>]>) {
	for job in jobs.iter().skip(WARMUP_TRIALS) {
		println!("{}", job.invocation_latency as f64 / 1_000.0);
	}
}

pub fn args() -> Result<(String, usize, usize), (i32, String)> {
	use std::env::args;

	let mut args = args();
	let prog = args.next().unwrap_or(String::from("<program>"));
	let usage = format!("USAGE: {} <svcname> [<numfuns> <numtrials>]", prog);

	let svcname = args.next().ok_or((1, usage.clone()))?;

	Ok(if let Some(numobjs) = args.next() {
		(
			svcname,
			numobjs.parse().or((Err((2, String::from("<numfuns>, if provided, must be a nonnegative integer")))))?,
			args.next().unwrap_or(usage.clone()).parse().or(Err((2, String::from("<numtrials>, if provided, must be a nonnegative integer"))))?,
		)
	} else {
		(svcname, 1, 1)
	})
}
