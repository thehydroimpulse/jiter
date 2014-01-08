use std::ptr;
use std::libc::*;
use std::libc;
use std::os;
use std::vec;
use std::fmt;

pub mod raw {
    use std::*;

    extern {
        fn mmap(addr : *libc::c_char, length : libc::size_t,
                prot : libc::c_int,   flags  : libc::c_int,
                fd   : libc::c_int,   offset : libc::off_t) -> *u8;
        fn munmap(addr : *u8, length : libc::size_t) -> libc::c_int;
        fn mprotect(addr: *libc::c_char, length: libc::size_t, prot: libc::c_int) -> libc::c_int;
    }

    /* From /usr/include/asm-generic/mman-common.h on Linux */

    /* prot values */
    pub static PROT_NONE   : libc::c_int = 0x0;
    pub static PROT_READ   : libc::c_int = 0x1;
    pub static PROT_WRITE  : libc::c_int = 0x2;
    pub static PROT_EXEC   : libc::c_int = 0x4;
    // ...

    /* flags */
    pub static MAP_SHARED  : libc::c_int = 0x1;
    pub static MAP_PRIVATE : libc::c_int = 0x2;
    // ...
}

struct MappedRegion {
  addr: *u8,
  len: libc::size_t
}

impl fmt::Default for MappedRegion {
  fn fmt(value: &MappedRegion, f: &mut std::fmt::Formatter) {
    write!(f.buf, "MappedRegion\\{{}, {}\\}", value.addr, value.len);
  }
}

impl Drop for MappedRegion {
  #[inline(never)]
  #[fixed_stack_segment]
  fn drop(&mut self) {
    unsafe {
      if raw::munmap(self.addr, self.len) < 0 {
        fail!(format!("munmap({}, {}): {}", self.addr, self.len, os::last_os_error()));
      }
    }
  }
}

#[inline(never)]
#[fixed_stack_segment]
unsafe fn mmap(size: size_t) -> Result<MappedRegion, ~str> {
  let buf = raw::mmap(0 as *libc::c_char, size,
    libc::PROT_READ | libc::PROT_WRITE,
    libc::MAP_PRIVATE | libc::MAP_ANON,
    -1, 0);
  return if buf == -1 as *u8 {
    Err(os::last_os_error())
  } else {
    Ok(MappedRegion{ addr: buf, len: size })
  }
}

#[fixed_stack_segment]
pub unsafe fn make_mem_exec(m: *u8, size: size_t) -> int {
  if raw::mprotect(m as *libc::c_char, size, libc::PROT_READ | PROT_EXEC) == -1 {
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

    let m = match mmap(1024) {
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
