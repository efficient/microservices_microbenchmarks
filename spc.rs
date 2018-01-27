#[link_section = ".data"]
#[no_mangle]
pub static mut SBOX: i64 = 0;

pub fn sbox() -> &'static mut i64 {
	unsafe {
		&mut SBOX
	}
}
