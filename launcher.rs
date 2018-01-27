mod job;
#[allow(dead_code)]
mod runtime;
mod time;

use job::args;
use job::joblist;
use job::printstats;
use runtime::LibFun;
use std::ffi::CString;
use std::process::exit;
use time::nsnow;

fn main() {
	let (svcname, numjobs) = args().unwrap_or_else(|(retcode, errmsg)| {
		eprintln!("{}", errmsg);
		exit(retcode);
	});
	let mut jobs = joblist(&mut |index| CString::new(format!("{}{}.so", svcname, index)).unwrap(), numjobs);

	for job in &mut *jobs {
		let fun = LibFun::new(&job.uservice_path).unwrap_or_else(|or| {
			eprintln!("{}", or);
			exit(2);
		});

		let ts = nsnow().unwrap();
		if let Some(fin) = fun() {
			job.invocation_latency = fin - ts;
		} else {
			let path = job.uservice_path.to_str().unwrap_or("");

			eprintln!("While invoking microservice: child '{}' died or was killed", path);
		}
	}

	printstats(&jobs);
}
