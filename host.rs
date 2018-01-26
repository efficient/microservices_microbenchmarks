#[allow(dead_code)]
mod ipc;
mod job;
mod time;

use ipc::SMem;
use job::Job;
use job::joblist;
use std::process::exit;
use time::nsnow;

const USERVICE_MASK: &str = "0x4";

fn main() {
	let (svcname, numjobs) = args().unwrap_or_else(|(retcode, errmsg)| {
		eprintln!("{}", errmsg);
		exit(retcode);
	});

	let mut jobs = joblist(&svcname, numjobs);
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

	for job in &*jobs {
		println!("{}", job.invocation_latency as f64 / 1_000.0);
	}
}

#[cfg(not(feature = "invoke_forkexec"))]
compile_error!("Must select an invoke_* personality via '--feature' or '--cfg feature='!");

#[cfg(feature = "invoke_forkexec")]
fn invoke(jobs: &mut Box<[Job]>, comms: &SMem<i64>) -> Result<(), String> {
	use std::process::Command;
	use std::process::Stdio;

	for job in &mut **jobs {
		let mut process = Command::new("taskset");
		process.arg(USERVICE_MASK).arg(&job.uservice_name).arg(format!("{}", comms.id()));
		if cfg!(debug_assertions) {
			process.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
		} else {
			process.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
		};

		let ts = nsnow().unwrap();
		let code = process.status().map_err(|msg| format!("{}: {}", job.uservice_name, msg))?;
		job.invocation_latency = **comms - ts;

		if ! code.success() {
			Err(if cfg!(debug_assertions) {
				let (stdout, stderr) = process.output().map(|both| (
					String::from_utf8_lossy(&both.stdout).into_owned(),
					String::from_utf8_lossy(&both.stderr).into_owned(),
				)).unwrap_or((String::new(), String::new()));

				format!("child '{}' died with {}\nChild's standard output:\nvvvvvvvvvvvvvvvvvvvvvvvv\n{}\n^^^^^^^^^^^^^^^^^^^^^^^^\nChild's standard error:\nvvvvvvvvvvvvvvvvvvvvvvv\n{}\n^^^^^^^^^^^^^^^^^^^^^^^", job.uservice_name, code, stdout, stderr)
			} else {
				format!("child '{}' died with {} [snip]", job.uservice_name, code)
			})?;
		}
	}

	Ok(())
}

fn args() -> Result<(String, usize), (i32, String)> {
	use std::env::args;

	let mut args = args();
	let prog = args.next().unwrap_or(String::from("<program>"));
	let usage = format!("USAGE: {} <svcname> [numjobs]", prog);

	Ok((
		args.next().ok_or((1, usage))?,
		args.next().unwrap_or(String::from("1")).parse().or(Err((2, String::from("[numjobs], if provided, must be a nonnegative integer"))))?,
	))
}
