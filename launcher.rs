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
use job::as_fixed_c_string;
use job::joblist;
use job::printstats;
use runtime::LibFun;
use std::process::exit;
use time::nsnow;

fn main() {
	let (svcname, numobjs, numjobs, _) = args("").unwrap_or_else(|(retcode, errmsg)| {
		println!("{}", errmsg);
		exit(retcode);
	});
	if numjobs < numobjs {
		println!("<numfuns> may not be greater than <numtrials>");
		exit(2);
	}

	if let Ok(shmid) = svcname.parse() {
		let mut job = SMem::from(shmid).unwrap_or_else(|msg| {
			eprintln!("{}", msg);
			exit(3);
		});

		loop {
			let &mut (ref mut ready, ref mut job) = &mut *job;
			if *ready {
				invoke(job, false);
				*ready = false;
			}
		}
	} else {
		let mut jobs = joblist(&mut |index| as_fixed_c_string(&format!("{}{}.so", svcname, index)), numobjs, numjobs);

		for job in &mut *jobs {
			invoke(job, true);
		}

		printstats(&jobs);
	}
}

fn invoke(job: &mut Job<FixedCString>, ts_before: bool) {
	let ts = if ts_before {
		nsnow().unwrap()
	} else {
		0
	};
	let fun = LibFun::new_from_ptr(job.uservice_path.as_ptr() as *const i8).unwrap_or_else(|or| {
		eprintln!("{}", or);
		exit(2);
	});

	if let Some(fin) = fun() {
		job.invocation_latency = fin - ts;
	} else {
		eprintln!("While invoking microservice: child '");
		for each in &job.uservice_path {
			if *each == b'\0' {
				break;
			}
			eprint!("{}", each);
		}
		eprintln!("' died or was killed");
	}
}
