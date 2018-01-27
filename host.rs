#[allow(dead_code)]
mod ipc;
mod job;
mod time;

use ipc::SMem;
use job::Job;
use job::args;
use job::joblist;
use job::printstats;
use std::process::exit;
use time::nsnow;

const USERVICE_MASK: &str = "0x4";

fn main() {
	let (svcname, numjobs) = args().unwrap_or_else(|(retcode, errmsg)| {
		eprintln!("{}", errmsg);
		exit(retcode);
	});

	let mut jobs = joblist(&mut |index| format!("{}{}", svcname, index), numjobs);
	let comm_handles = if cfg!(feature = "invoke_forkexec") {
		SMem::new(0i64).unwrap_or_else(|error| {
			eprintln!("Initializing shared memory: {}", error);
			exit(3);
		})
	} else {
		unreachable!();
	};

	if let Err(or) = invoke(&mut jobs, &comm_handles) {
		eprintln!("While invoking microservice: {}", or);
		exit(4);
	}

	printstats(&jobs);
}

#[cfg(not(feature = "invoke_forkexec"))]
compile_error!("Must select an invoke_* personality via '--feature' or '--cfg feature='!");

#[cfg(feature = "invoke_forkexec")]
fn invoke(jobs: &mut Box<[Job<String>]>, comms: &SMem<i64>) -> Result<(), String> {
	use std::process::Command;
	use std::process::Stdio;

	for job in &mut **jobs {
		let mut process = Command::new("taskset");
		process.arg(USERVICE_MASK).arg(&job.uservice_path).arg(format!("{}", comms.id()));
		if cfg!(debug_assertions) {
			process.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
		} else {
			process.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
		};

		let ts = nsnow().unwrap();
		let code = process.status().map_err(|msg| format!("{}: {}", job.uservice_path, msg))?;
		job.invocation_latency = nsnow().unwrap() - ts;

		if ! code.success() {
			Err(if cfg!(debug_assertions) {
				let (stdout, stderr) = process.output().map(|both| (
					String::from_utf8_lossy(&both.stdout).into_owned(),
					String::from_utf8_lossy(&both.stderr).into_owned(),
				)).unwrap_or((String::new(), String::new()));

				format!("child '{}' died with {}\nChild's standard output:\nvvvvvvvvvvvvvvvvvvvvvvvv\n{}\n^^^^^^^^^^^^^^^^^^^^^^^^\nChild's standard error:\nvvvvvvvvvvvvvvvvvvvvvvv\n{}\n^^^^^^^^^^^^^^^^^^^^^^^", job.uservice_path, code, stdout, stderr)
			} else {
				format!("child '{}' died with {} [snip]", job.uservice_path, code)
			})?;
		}
	}

	Ok(())
}
