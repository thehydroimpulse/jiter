use std::ptr;
use std::libc::*;
use std::libc;
use std::os;
use std::vec;
use std::fmt;

struct MappedRegion {
  addr: *libc::c_void,
  len: libc::size_t
}

impl fmt::Default for MappedRegion {
  fn fmt(value: &MappedRegion, f: &mut std::fmt::Formatter) {
    write!(f.buf, "MappedRegion\\{{}, {}\\}", value.addr, value.len);
  }
}

impl Drop for MappedRegion {
  #[fixed_stack_segment]
  fn drop(&mut self) {
    debug!(format!("munmapping {}", self.addr));
    unsafe {
      if libc::munmap(self.addr, self.len) < 0 {
        fail!(format!("munmap({}, {}): {}", self.addr, self.len, os::last_os_error()));
      } else {
        self.addr = 0 as *libc::c_void;
        self.len  = 0;
      }
    }
  }
}

#[fixed_stack_segment]
fn mmap(size: size_t) -> Result<MappedRegion, ~str> {
  unsafe {
    let ptr = libc::mmap(0 as *libc::c_void, size,
      libc::PROT_READ | libc::PROT_WRITE,
      libc::MAP_PRIVATE | libc::MAP_ANON,
      -1, 0) as *libc::c_void;
    return if ptr == libc::MAP_FAILED {
      Err(os::last_os_error())
    } else {
      Ok(MappedRegion{ addr: ptr, len: size })
    }
  }
}

#[fixed_stack_segment]
pub unsafe fn make_mem_exec(m: *c_void, size: size_t) -> int {
  if mprotect(m, size, libc::PROT_READ | PROT_EXEC) == -1 {
    fail!("err: mprotect");
  }

  return 0;
}

// Guessing the bus error is here.
#[fixed_stack_segment]
pub unsafe fn emit_code(src: &[u8], mem: &MappedRegion) {
  ptr::copy_memory(mem.addr as *mut c_void, vec::raw::to_ptr(src) as *mut c_void, src.len());
}

fn add(a: u32) -> u32 {
  a + 4
}


fn main() {

  // 0:  c6 03 02  mov    BYTE PTR [ebx],2
  // 3:  83 c3 05  add    ebx,5
  let code = [
    0xC6, 0x00, 0x05, 0x83, 0xC0, 0x0A, 0xC3
  ];

  unsafe {

    let m = match mmap(40) {
      Ok(r) => r,
      Err(s) => fail!("err: %?", s)
    };

    emit_code(code, &m);
    make_mem_exec(m.addr, m.len);

    let mut JITFn: extern fn(num: u32) -> u32;
    JITFn = m.addr as extern fn(num: u32) -> u32;
    println!("Function: {}", m);
    let result = JITFn(2);
    println(fmt!("result: {%?}", result as int));
    println("Works");
  }
}
