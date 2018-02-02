#[cfg(feature = "invoke_sendmsg")]
mod bytes;
#[allow(dead_code)]
#[cfg(any(feature = "invoke_forkexec", feature = "invoke_launcher"))]
mod ipc;
#[cfg_attr(not(feature = "invoke_launcher"), allow(dead_code))]
mod job;
#[cfg_attr(not(feature = "invoke_sendmsg"), allow(dead_code))]
mod pgroup;
#[allow(dead_code)]
#[cfg(any(feature = "invoke_sendmsg", feature = "invoke_launcher"))]
mod ringbuf;
mod time;

#[cfg(feature = "invoke_sendmsg")]
use bytes::Bytes;
#[cfg(any(feature = "invoke_forkexec", feature = "invoke_launcher"))]
use ipc::SMem;
#[cfg(feature = "invoke_launcher")]
use job::FixedCString;
use job::Job;
use job::args;
use job::printstats;
use pgroup::exit;
#[cfg(any(feature = "invoke_sendmsg", feature = "invoke_launcher"))]
use pgroup::kill_at_exit;
#[cfg(any(feature = "invoke_sendmsg", feature = "invoke_launcher"))]
use pgroup::setpgid;
#[cfg(feature = "invoke_launcher")]
use pgroup::term;
#[cfg(any(feature = "invoke_sendmsg", feature = "invoke_launcher"))]
use ringbuf::RingBuffer;
use std::cell::Cell;
use std::cell::RefCell;
use std::env::Args;
use std::fmt::Display;
#[cfg(feature = "invoke_launcher")]
use std::iter::repeat;
use std::mem::replace;
#[cfg(feature = "invoke_sendmsg")]
use std::net::SocketAddr;
#[cfg(feature = "invoke_sendmsg")]
use std::net::UdpSocket;
#[cfg(any(feature = "invoke_sendmsg", feature = "invoke_launcher"))]
use std::os::unix::process::CommandExt;
#[cfg(any(feature = "invoke_sendmsg", feature = "invoke_launcher"))]
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
#[cfg(feature = "invoke_launcher")]
use std::sync::atomic::AtomicBool;
#[cfg(feature = "invoke_launcher")]
use std::sync::atomic::Ordering;
use time::nsnow;

const DEFAULT_USERVICE_MASK: &str = "0x4";

#[cfg(not(feature = "invoke_launcher"))]
const USAGE_EXTENDED: &str = "[cpumask]";
#[cfg(feature = "invoke_launcher")]
const USAGE_EXTENDED: &str = "[cpumask] [quantum] [limit]";

thread_local! {
	static ATTACH_STREAMS: Cell<bool> = Cell::new(false);
	static USERVICE_MASK: RefCell<String> = RefCell::new(String::from(DEFAULT_USERVICE_MASK));
}

fn main() {
	let (svcname, numobjs, numjobs, attach_streams, mut args) = args(USAGE_EXTENDED).unwrap_or_else(|(retcode, errmsg)| {
		println!("{}", errmsg);
		exit(retcode);
	});
	if numjobs < numobjs {
		println!("<numfuns> may not be greater than <numtrials>");
		exit(2);
	}
	ATTACH_STREAMS.with(|streams| streams.set(attach_streams));
	if let Some(mask) = args.next() {
		if &mask[0..2] != "0x" {
			println!("[cpumask], if provided, must be a hex mask starting with '0x'");
			exit(2);
		}

		USERVICE_MASK.with(|uservice_mask| replace(&mut *uservice_mask.borrow_mut(), mask));
	}

	let mut jobs = joblist(&svcname, numobjs, numjobs);
	let mut comm_handles = handshake(&jobs, numobjs, &mut args).unwrap_or_else(|msg| {
		eprintln!("During initialization: {}", msg);
		exit(3);
	});

	if let Err(or) = invoke(&mut jobs, &mut comm_handles) {
		eprintln!("While invoking microservice: {}", or);
		exit(4);
	}

	if ! attach_streams {
		printstats(&jobs);
	}
}

#[cfg(not(any(feature = "invoke_forkexec", feature = "invoke_sendmsg", feature = "invoke_launcher")))]
compile_error!("Must select an invoke_* personality via '--feature' or '--cfg feature='!");

#[cfg(feature = "invoke_sendmsg")]
type Comms = (UdpSocket, RingBuffer<(Child, SocketAddr)>);

#[cfg(feature = "invoke_launcher")]
type Comms<'a> = RingBuffer<(Child, SMem<'a, (AtomicBool, Job<FixedCString>)>)>;

#[cfg(not(feature = "invoke_launcher"))]
pub fn joblist(svcname: &str, numobjs: usize, numjobs: usize) -> Box<[Job<String>]> {
	use job::joblist;

	joblist(|index| format!("{}{}", svcname, index), numobjs, numjobs)
}

#[cfg(feature = "invoke_launcher")]
pub fn joblist(svcname: &str, numobjs: usize, numjobs: usize) -> Box<[Job<FixedCString>]> {
	use job::joblist;

	joblist(|index| {
		FixedCString::from(&format!("{}{}.so", svcname, index))
	}, numobjs, numjobs)
}

#[cfg(feature = "invoke_forkexec")]
fn handshake<'a>(_: &Box<[Job<String>]>, _: usize, _: &mut Args) -> Result<SMem<'a, i64>, String> {
	SMem::new(0).map_err(|or| format!("Initializing shared memory: {}", or))
}

#[cfg(feature = "invoke_sendmsg")]
fn handshake(jobs: &Box<[Job<String>]>, nprocs: usize, _: &mut Args) -> Result<Comms, String> {
	const BATCH_SIZE: usize = 100;

	let socket = UdpSocket::bind("127.0.0.1:0").map_err(|or| format!("Initializing UDP socket: {}", or))?;
	let addr = socket.local_addr().map_err(|or| format!("Determining socket address: {}", or))?;

	let mut pgroup = 0;
	let handles: Vec<_> = (0..nprocs / BATCH_SIZE + 1).flat_map(|group| {
		let procs: Vec<_> = (group * BATCH_SIZE..nprocs.min((group + 1) * BATCH_SIZE)).map(|job| {
			let mut handle = process(&jobs[job].uservice_path, &format!("{} 127.0.0.{}:0 {}", addr, group + 2, job));
			handle.before_exec(move || setpgid(pgroup).map(|_| ()));

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

	Ok((socket, RingBuffer::new(handles.into_boxed_slice())))
}

#[cfg(feature = "invoke_launcher")]
fn handshake<'a>(_: &Box<[Job<FixedCString>]>, nlibs: usize, args: &mut Args) -> Result<Comms<'a>, String> {
	let ones = USERVICE_MASK.with(|uservice_mask| {
		usize::from_str_radix(&uservice_mask.borrow()[2..], 16).unwrap().count_ones()
	});

	let mut pgroup = 0;
	let them: Vec<_> = (0..ones).map(|count| {
		let mem = SMem::new((AtomicBool::new(false), Job::new(FixedCString::new()))).unwrap_or_else(|msg| {
			eprintln!("Initializing shared memory: {}", msg);
			exit(5);
		});

		let mut handle = process("./launcher", format!("{}", mem.id()));
		for arg in args.chain(repeat(String::from("0"))).take(2) {
			handle.arg(arg);
		}
		handle.before_exec(move || setpgid(pgroup).map(|_| ()));
		let handle = handle.spawn().unwrap_or_else(|msg| {
			eprintln!("Spawning launcher process: {}", msg);
			exit(6);
		});
		if count == 0 {
			pgroup = handle.id();
			kill_at_exit(-(pgroup as i32));
		}

		(handle, mem)
	}).collect();

	Ok(RingBuffer::with_alignment(them.into_boxed_slice(), nlibs))
}

#[cfg(feature = "invoke_forkexec")]
fn invoke(jobs: &mut Box<[Job<String>]>, comms: &SMem<i64>) -> Result<(), String> {
	for job in &mut **jobs {
		let mut process = process(&job.uservice_path, comms.id());

		let ts = nsnow().unwrap();
		let code = process.status().map_err(|msg| format!("{}: {}", job.uservice_path, msg))?;
		job.completion_time = nsnow().unwrap() - ts;
		job.invocation_latency = **comms - ts;

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
		jobs[job].completion_time = nsnow().unwrap() - sta;
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

#[cfg(feature = "invoke_launcher")]
fn invoke(jobs: &mut Box<[Job<FixedCString>]>, comms: &mut Comms) -> Result<(), String> {
	for job in 0..jobs.len() {
		let &mut (_, ref mut task) = &mut comms[job];
		task.1 = jobs[job].clone();

		let ts = nsnow().unwrap();
		*task.0.get_mut() = true;
		while task.0.load(Ordering::Relaxed) {}
		jobs[job].completion_time = nsnow().unwrap() - ts;
		jobs[job].invocation_latency = task.1.invocation_latency - ts;
	}

	for &mut (ref mut launcher, _) in &mut **comms {
		term(launcher.id() as i32).map_err(|err| format!("Terminating child: {}", err))?;
	}

	for &mut (ref mut launcher, _) in &mut **comms {
		launcher.wait().map_err(|err| format!("Waiting on child: {}", err))?;
	}

	Ok(())
}

fn process<T: Display>(path: &str, arg: T) -> Command {
	let mut process = Command::new("taskset");

	USERVICE_MASK.with(|uservice_mask| process.arg(&*uservice_mask.borrow()));
	process.arg(path).arg(format!("{}", arg));

	process.stdin(Stdio::null());
	if ! ATTACH_STREAMS.with(|attach_streams| attach_streams.get()) {
		if cfg!(debug_assertions) {
			process.stdout(Stdio::piped()).stderr(Stdio::piped());
		} else {
			process.stdout(Stdio::null()).stderr(Stdio::null());
		}
	}

	process
}
