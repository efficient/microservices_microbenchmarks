const NUM_USERVICES: usize = 10000;

pub struct Job {
	pub uservice_name: String,
	pub invocation_latency: i64,
}

pub fn joblist(svcname: &str, numjobs: usize) -> Box<[Job]> {
	match numjobs {
		1 => Box::new([Job {
			uservice_name: String::from(svcname),
			invocation_latency: 0,
		}]),
		_ => {
			let list: Vec<_> = (0..numjobs).map(|index| Job {
				uservice_name: format!("{}{}", svcname, index % NUM_USERVICES),
				invocation_latency: 0,
			}).collect();

			list.into_boxed_slice()
		},
	}
}
