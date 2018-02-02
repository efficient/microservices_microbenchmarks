use std::time::UNIX_EPOCH;
use std::time::SystemTime;
use std::time::SystemTimeError;

pub fn nsnow() -> Result<i64, SystemTimeError> {
	let time = SystemTime::now().duration_since(UNIX_EPOCH)?;

	Ok((time.as_secs() * 1_000_000_000 + time.subsec_nanos() as u64) as i64)
}
