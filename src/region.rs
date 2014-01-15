use std::os;
use std;
mod raw;

pub struct MappedRegion {
    addr: *u8,
    len: u64
}

impl std::fmt::Default for MappedRegion {
    fn fmt(value: &MappedRegion, f: &mut std::fmt::Formatter) {
        write!(f.buf, "MappedRegion\\{{}, {}\\}", value.addr, value.len);
    }
}

impl Drop for MappedRegion {
    #[inline(never)]
    fn drop(&mut self) {
        unsafe {
            if raw::munmap(self.addr, self.len) < 0 {
                fail!(format!("munmap({}, {}): {}", self.addr, self.len, os::last_os_error()));
            }
        }
    }
}