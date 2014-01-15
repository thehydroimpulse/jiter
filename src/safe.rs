#[allow(unused_imports)];
#[allow(unused_variable)];

use region::MappedRegion;
use std::libc::{c_char, size_t, c_void};
use std::libc;
use std::os;

mod raw;
mod region;

/**
 * Provide a safe interface to the native `mmap` function.
 *
 * @safe
 * @param {u64} size
 * @return {Result<MappedRegion, ~str>}
 */

pub fn mmap(size: u64) -> Result<~MappedRegion, ~str> {
    unsafe {
        let buf = raw::mmap(
            0 as *libc::c_char,
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON,
            -1,
            0
        );

        if buf == -1 as *u8 {
          Err(os::last_os_error())
        } else {
          Ok(~MappedRegion{ addr: buf, len: size })
        }
    }
}

/**
 * Provide a safe interface to the native `memcpy` function.
 *
 * @safe
 * @param {&MappedRegion} region
 * @param {&[u8]} contents
 */

pub fn memcpy(region: &MappedRegion, contents: *u8) {
    unsafe {
        raw::memcpy(
            region.addr as * c_void,
            contents as *c_void,
            region.len as size_t);
        assert_eq!(*contents, *region.addr);
    }
}

pub fn mprotect(region: &MappedRegion, contents: &[u8]) {
    unsafe {
        if raw::mprotect(
            region.addr as *libc::c_void,
            contents.len() as libc::size_t,
            libc::PROT_READ | libc::PROT_EXEC
        ) == -1 {
            fail!("err: mprotect failed to protect the memory region.");
        }
    }
}