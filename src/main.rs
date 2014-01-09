#[crate_id = "jiter#0.0.1"];
#[desc = "Jiter"];
#[crate_type = "bin"];
#[license = "MIT"];
#[warn(non_camel_case_types)];
#[feature(globs)];

use std::ptr;
use std::libc::{c_char, size_t, c_void, PROT_EXEC};
use std::libc;
use std::os;
use std::cast;

#[no_mangle]
type JitFn = extern "C" fn(n: int) -> int;

pub mod raw {
    use std::libc;
    extern {
        pub fn mmap(addr : *libc::c_char, length : libc::size_t,
                prot : libc::c_int,   flags  : libc::c_int,
                fd   : libc::c_int,   offset : libc::off_t) -> *u8;
        pub fn munmap(addr : *u8, length : libc::size_t) -> libc::c_int;
        pub fn mprotect(addr: *libc::c_char, length: libc::size_t, prot: libc::c_int) -> libc::c_int;
        pub fn memcpy(dest: *libc::c_void, src: *libc::c_void, n: libc::size_t) -> *libc::c_void;
    }

    pub static PROT_NONE   : libc::c_int = 0x0;
    pub static PROT_READ   : libc::c_int = 0x1;
    pub static PROT_WRITE  : libc::c_int = 0x2;
    pub static PROT_EXEC   : libc::c_int = 0x4;

    pub static MAP_SHARED  : libc::c_int = 0x1;
    pub static MAP_PRIVATE : libc::c_int = 0x2;
}

struct MappedRegion {
    addr: *u8,
    len: libc::size_t
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

#[inline(never)]
unsafe fn mmap(size: size_t) -> Result<MappedRegion, ~str> {
    let buf = raw::mmap(0 as *libc::c_char, size,
        libc::PROT_READ | libc::PROT_WRITE,
        libc::MAP_PRIVATE | libc::MAP_ANON, -1, 0);

    return if buf == -1 as *u8 {
        Err(os::last_os_error())
    } else {
        Ok(MappedRegion{ addr: buf, len: size })
    }
}

pub unsafe fn make_mem_exec(m: *u8, size: size_t) -> int {
    if raw::mprotect(m as *libc::c_char, size, libc::PROT_READ | PROT_EXEC) == -1 {
        fail!("err: mprotect");
    }

    return 0;
}

// Guessing the bus error is here.
pub unsafe fn emit_code(src: *u8, len: uint, mem: &MappedRegion) {
    ptr::copy_memory(mem.addr as *mut c_void, src as *mut c_void, len);
}

fn main() {

  let code = [
    0x48, 0x89, 0xf8,       // mov %rdi, %rax
    0x48, 0x83, 0xc0, 0x04, // add $4, %rax
    0xc3                    // ret
  ];

  unsafe {

        //let m = match mmap(1024) {
            // Ok(r) => r,
            // Err(s) => fail!("err: {}", s)
        //};

        let buf = raw::mmap(0 as *libc::c_char, code.len() as u64, libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON, -1, 0);

        if buf == -1 as *u8 {
            fail!(os::last_os_error());
        }

        println("copying the memory contents.");
        raw::memcpy(buf as * c_void, code.as_ptr() as *c_void, code.len() as size_t);

        println!("original: {} mmapped: {}", *(code.as_ptr()), *buf);

        // Check the mmapped region contains the exact correct contents.
        assert!(*(code.as_ptr()) == *buf);

        println("protecting the mmapped region.");
        if raw::mprotect(buf as *libc::c_char, code.len() as u64, libc::PROT_READ | PROT_EXEC) == -1 {
            fail!("err: mprotect");
        }

        //emit_code(code.as_ptr(), code.len(), &m);
        //make_mem_exec(m.addr, m.len);

        let func: JitFn = cast::transmute(buf);
        //JITFn = m.addr as |num: u32| -> u32;
        //println!("Function: {}", m);
        let value = func(5);
        //println!("result: {}", result);
        println!("func(): {}", value);
    }
}
