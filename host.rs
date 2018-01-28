#[cfg(feature = "invoke_sendmsg")]
mod bytes;
#[allow(dead_code)]
#[cfg(feature = "invoke_forkexec")]
mod ipc;
mod job;
#[cfg_attr(not(feature = "invoke_sendmsg"), allow(dead_code))]
mod pgroup;
mod time;

#[cfg(feature = "invoke_sendmsg")]
use bytes::Bytes;
#[cfg(feature = "invoke_forkexec")]
use ipc::SMem;
use job::Job;
use job::args;
use job::joblist;
use job::printstats;
use pgroup::exit;
#[cfg(feature = "invoke_sendmsg")]
use pgroup::kill_at_exit;
use std::fmt::Display;
#[cfg(feature = "invoke_sendmsg")]
use std::os::unix::process::CommandExt;
#[cfg(feature = "invoke_sendmsg")]
use std::net::SocketAddr;
#[cfg(feature = "invoke_sendmsg")]
use std::net::UdpSocket;
#[cfg(feature = "invoke_sendmsg")]
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use time::nsnow;

const USERVICE_MASK: &str = "0x4";

fn main() {
	let (svcname, numjobs) = args().unwrap_or_else(|(retcode, errmsg)| {
		eprintln!("{}", errmsg);
		exit(retcode);
	});

	let mut jobs = joblist(&mut |index| format!("{}{}", svcname, index), numjobs);
	let mut comm_handles = handshake(&jobs).unwrap_or_else(|msg| {
		eprintln!("During initialization: {}", msg);
		exit(3);
	});

	if let Err(or) = invoke(&mut jobs, &mut comm_handles) {
		eprintln!("While invoking microservice: {}", or);
		exit(4);
	}

	printstats(&jobs);
}

#[cfg(not(any(feature = "invoke_forkexec", feature = "invoke_sendmsg")))]
compile_error!("Must select an invoke_* personality via '--feature' or '--cfg feature='!");

#[cfg(feature = "invoke_sendmsg")]
type Comms = (UdpSocket, Box<[(Child, SocketAddr)]>);

#[cfg(feature = "invoke_forkexec")]
fn handshake<'a>(_: &Box<[Job<String>]>) -> Result<SMem<'a, i64>, String> {
	SMem::new(0).map_err(|or| format!("Initializing shared memory: {}", or))
}

#[cfg(feature = "invoke_sendmsg")]
fn handshake(jobs: &Box<[Job<String>]>) -> Result<Comms, String> {
	const BATCH_SIZE: usize = 100;

	let socket = UdpSocket::bind("127.0.0.1:0").map_err(|or| format!("Initializing UDP socket: {}", or))?;
	let addr = socket.local_addr().map_err(|or| format!("Determining socket address: {}", or))?;

	let mut pgroup = 0;
	let handles: Vec<_> = (0..jobs.len() / BATCH_SIZE + 1).flat_map(|group| {
		let procs: Vec<_> = (group * BATCH_SIZE..jobs.len().min((group + 1) * BATCH_SIZE)).map(|job| {
			let mut handle = process(&jobs[job].uservice_path, &format!("{} 127.0.0.{}:0 {}", addr, group + 2, job));
			handle.gid(pgroup);

			let handle = handle.spawn().unwrap_or_else(|msg| {
				eprintln!("Spawning child process '{}': {}", jobs[job].uservice_path, msg);
				exit(5);
			});
			if job == 0 {
				pgroup = handle.id();
				kill_at_exit(-(pgroup as i32));
			}

			handle
		}).collect();

		let mut ports: Vec<_> = (0..procs.len()).map(|_| {
			let mut process = 0usize;
			let (_, addr) = socket.recv_from(process.bytes()).unwrap_or_else(|msg| {
				eprintln!("Socket handshake: {}", msg);
				exit(6);
			});

			(process, addr)
		}).collect();
		ports.sort_by_key(|&(process, _)| process);

		procs.into_iter().zip(ports.into_iter().map(|(_, addr)| addr))
	}).collect();

	Ok((socket, handles.into_boxed_slice()))
}

#[cfg(feature = "invoke_forkexec")]
fn invoke(jobs: &mut Box<[Job<String>]>, comms: &SMem<i64>) -> Result<(), String> {
	for job in &mut **jobs {
		let mut process = process(&job.uservice_path, comms.id());

		let ts = nsnow().unwrap();
		let code = process.status().map_err(|msg| format!("{}: {}", job.uservice_path, msg))?;
		job.invocation_latency = nsnow().unwrap() - ts;

		if ! code.success() {
			Err(if cfg!(debug_assertions) {
				let (stdout, stderr) = process.output().map(|both| (
					String::from_utf8_lossy(&both.stdout).into_owned(),
					String::from_utf8_lossy(&both.stderr).into_owned(),
				)).unwrap_or((String::new(), String::new()));

				format!("Child '{}' died with {}\nChild's standard output:\nvvvvvvvvvvvvvvvvvvvvvvvv\n{}\n^^^^^^^^^^^^^^^^^^^^^^^^\nChild's standard error:\nvvvvvvvvvvvvvvvvvvvvvvv\n{}\n^^^^^^^^^^^^^^^^^^^^^^^", job.uservice_path, code, stdout, stderr)
			} else {
				format!("Child '{}' died with {} [snip]", job.uservice_path, code)
			})?;
		}
	}

	Ok(())
}

#[cfg(feature = "invoke_sendmsg")]
fn invoke(jobs: &mut Box<[Job<String>]>, comms: &mut Comms) -> Result<(), String> {
	let &mut (ref me, ref mut them) = comms;

	for job in 0..jobs.len() {
		let (_, addr) = them[job];

		let mut fin = 0i64;
		let sta = nsnow().unwrap();
		me.send_to(().bytes(), &addr).map_err(|err| format!("Sending to child {}: {}", job, err))?;
		me.recv(fin.bytes()).map_err(|err| format!("Receiving from child {}: {}", job, err))?;
		jobs[job].invocation_latency = fin - sta;
	}

	for &mut (ref mut child, _) in &mut **them {
		child.kill().map_err(|err| format!("Killing child: {}", err))?;
	}

	for &mut (ref mut child, _) in &mut **them {
		child.wait().map_err(|err| format!("Waiting on child: {}", err))?;
	}

	Ok(())
}

fn process<T: Display>(path: &str, arg: T) -> Command {
	let mut process = Command::new("taskset");

	process.arg(USERVICE_MASK).arg(path).arg(format!("{}", arg));
	if cfg!(debug_assertions) {
		process.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
	} else {
		process.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
	};

	process
}
