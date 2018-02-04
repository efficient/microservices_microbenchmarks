#[allow(dead_code)]
mod ipc;
mod job;
#[allow(dead_code)]
mod runtime;
mod time;

use ipc::SMem;
use job::FixedCString;
use job::Job;
use job::args;
use job::joblist;
use job::printstats;
use runtime::LibFun;
use runtime::query_preemption;
use runtime::setup_preemption;
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use time::nsnow;

fn main() {
	let (svcname, numobjs, numjobs, _, _) = args("").unwrap_or_else(|(retcode, errmsg)| {
		println!("{}", errmsg);
		exit(retcode);
	});

	if let Ok(shmid) = svcname.parse() {
		let mut job = SMem::from(shmid).unwrap_or_else(|msg| {
			eprintln!("Setting up shared memory: {}", msg);
			exit(3);
		});
		let quantum = numobjs as i64;
		let limit = if numjobs == 0 { i64::max_value() } else { numjobs as i64 * 1_000 };

		let mut ts = nsnow().unwrap();
		if quantum != 0 {
			if let Err(or) = setup_preemption(quantum, limit, &ts) {
				eprintln!("Setting up preemption: {}", or);
				exit(4);
			}
		}
		let mut preemptions = None;
		while preemptions.is_none() {
			let &mut (ref mut ready, ref mut job): &mut (AtomicBool, _) = &mut *job;
			if ready.load(Ordering::Relaxed) {
				invoke(job, &mut ts, false);
				*ready.get_mut() = false;
			}

			preemptions = query_preemption();
		}
		let preemptions = preemptions.unwrap();

		if preemptions.iter().any(|nonzero| *nonzero != 0) {
			for preemption in preemptions {
				println!("{}", preemption.abs() as f64 / 1_000.0);
			}
		}
	} else {
		if numjobs < numobjs {
			println!("<numfuns> may not be greater than <numtrials>");
			exit(2);
		}

		let mut ts = 0;
		let mut jobs = joblist(|index| FixedCString::from(&format!("{}{}.so", svcname, index)), numobjs, numjobs);

		for job in &mut *jobs {
			invoke(job, &mut ts, true);
		}

		printstats(&jobs, 0.0);
	}
}

#[cfg(not(feature = "memoize_loaded"))]
fn invoke(job: &mut Job<FixedCString>, ts: &mut i64, ts_before: bool) {
	let mut fun = LibFun::new_from_ptr(job.uservice_path.as_ptr() as *const i8).unwrap_or_else(|msg| {
		eprintln!("{}", msg);
		exit(2);
	});

	call(job, ts, &mut *fun, ts_before);
}

#[cfg(feature = "memoize_loaded")]
fn invoke(job: &mut Job<FixedCString>, ts: &mut i64, ts_before: bool) {
	use std::cell::RefCell;
	use std::collections::HashMap;

	thread_local! {
		static MEMO: RefCell<HashMap<FixedCString, LibFun>> = RefCell::new(HashMap::new());
	}

	MEMO.with(|memo| {
		let mut memo = memo.borrow_mut();
		let fun = memo.entry(job.uservice_path.clone()).or_insert_with(|| LibFun::new_from_ptr(job.uservice_path.as_ptr() as *const i8).unwrap_or_else(|or| {
			eprintln!("{}", or);
			exit(2);
		}));

		call(job, ts, &mut **fun, ts_before);
	});
}

fn call<T: FnMut(i64) -> Option<i64>>(job: &mut Job<FixedCString>, ts: &mut i64, mut fun: T, ts_before: bool) {
	*ts = nsnow().unwrap();
	let ts = if ts_before {
		*ts
	} else {
		0
	};
	if let Some(fin) = fun(0) {
		job.invocation_latency = fin - ts;
	} else {
		eprintln!("While invoking microservice: child '");
		for each in &*job.uservice_path {
			if *each == b'\0' {
				break;
			}
			eprint!("{}", each);
		}
		eprintln!("' died or was killed");
	}
}

#[cfg(not(any(feature = "cleanup_loaded", feature = "memoize_loaded", feature = "preserve_loaded")))]
compile_error!("Must select an *_loaded personality via '--feature' or '--cfg feature='!");
