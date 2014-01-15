#[crate_id = "jiter#0.0.1"];
#[desc = "Jiter"];
#[crate_type = "bin"];
#[license = "MIT"];
#[feature(macro_rules)];
#[allow(dead_code)];

use std::libc::{c_char, size_t, c_void, PROT_EXEC};
use std::libc;
use std::os;
use std::cast;
use region::MappedRegion;

mod raw;
mod region;

type JitFn = extern "C" fn(n: int) -> int;

/**
 * Provide a safe interface to the native `memcpy` function.
 *
 * @safe
 * @param {&MappedRegion} region
 * @param {&[u8]} contents
 */

pub fn safe_memcpy(region: &MappedRegion, contents: &[u8]) {
    unsafe {
        raw::memcpy(
            region.addr as * c_void,
            contents.as_ptr() as *c_void,
            contents.len() as size_t);
    }
}

#[test]
fn test_safe_memcpy() {
    let contents = [0x48, 0x0c];

    let region = match safe_mmap(contents.len() as u64) {
        Ok(r) => r,
        Err(err) => fail!(err)
    };

    safe_memcpy(&region, contents);

    // Check the contents of the new mapped memory region.
    unsafe {
        assert!(*(contents.as_ptr()) == *region.addr);
    }
}

/**
 * Provide a safe interface to the native `mmap` function.
 *
 * @safe
 * @param {u64} size
 * @return {Result<MappedRegion, ~str>}
 */

fn safe_mmap(size: u64) -> Result<MappedRegion, ~str> {
    unsafe {
        let buf = raw::mmap(0 as *libc::c_char, size, libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON, -1, 0);

        if buf == -1 as *u8 {
            Err(os::last_os_error())
        } else {
            Ok(MappedRegion{ addr: buf, len: size })
        }
    }
}

fn safe_mem_protect(region: &MappedRegion, contents: &[u8]) {
    unsafe {
        if raw::mprotect(
            region.addr as *libc::c_char,
            contents.len() as u64,
            libc::PROT_READ | PROT_EXEC
        ) == -1 {
            fail!("err: mprotect failed to protect the memory region.");
        }
    }
}

/**
 * JIT a function dynamically. This will compile the contents (x86 instructions)
 * and return a function that you can call normally.
 *
 * @safe
 * @param {&[u8]} contents
 */

fn jit_func<F>(contents: &[u8]) -> F {
    let region = match safe_mmap(contents.len() as u64) {
        Ok(r) => r,
        Err(err) => fail!(err)
    };

    safe_memcpy(&region, contents);

    // Check the contents of the new mapped memory region.
    unsafe {
        assert_eq!(*(contents.as_ptr()), *region.addr);
    }

    safe_mem_protect(&region, contents);

    unsafe {
        let func: F = cast::transmute(region.addr);
        return func;
    }
}

#[test]
fn test_jit_func() {
    let contents = [
        0x48, 0x89, 0xf8,       // mov %rdi, %rax
        0x48, 0x83, 0xc0, 0x04, // add $4, %rax
        0xc3                    // ret
    ];

    type Func = extern "C" fn(n: int) -> int;
    let func = jit_func::<JitFn>(contents);

    assert_eq!(func(4), 8);
}


#[test]
fn test_safe_mem_protect() {
    let contents = [
        0x48, 0x89, 0xf8,       // mov %rdi, %rax
        0x48, 0x83, 0xc0, 0x04, // add $4, %rax
        0xc3                    // ret
    ];

    let region = match safe_mmap(contents.len() as u64) {
        Ok(r) => r,
        Err(err) => fail!(err)
    };

    safe_memcpy(&region, contents);

    // Check the contents of the new mapped memory region.
    unsafe {
        assert!(*(contents.as_ptr()) == *region.addr);
    }

    safe_mem_protect(&region, contents);
}

#[test]
fn test_safe_mmap() {
    let contents = [0,1,2];

    match safe_mmap(contents.len() as u64) {
        Ok(_) => assert!(true),
        Err(err) => fail!(err)
    }
}

fn main() {

    let code = [
        0x48, 0x89, 0xf8,       // mov %rdi, %rax
        0x48, 0x83, 0xc0, 0x04, // add $4, %rax
        0xc3                    // ret
    ];

    let region = match safe_mmap(code.len() as u64) {
        Ok(r) => r,
        Err(err) => fail!(err)
    };

    unsafe {

        let buf = region.addr;

        //raw::memcpy(buf as * c_void, code.as_ptr() as *c_void, code.len() as size_t);

        //println!("original: {} mmapped: {}", *(code.as_ptr()), *buf);

        // Check the mmapped region contains the exact correct contents.
        //assert!(*(code.as_ptr()) == *buf);

        //println("protecting the mmapped region.");
        //if raw::mprotect(buf as *libc::c_char, code.len() as u64, libc::PROT_READ | PROT_EXEC) == -1 {
        //    fail!("err: mprotect");
        //}

        let func: JitFn = cast::transmute(buf);
        let value = func(5);

        println!("func(): {}", value);

        println!("munmapping memory region: {}", buf);
        // Free the mmapped memory page:
        if raw::munmap(buf, code.len() as u64) < 0 {
            fail!(format!("munmap({}, {}): {}", buf, code.len(), os::last_os_error()));
        }
    }
}
