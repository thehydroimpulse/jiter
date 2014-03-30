#[crate_id = "jiter#0.0.1"];
#[desc = "Jiter"];
#[crate_type = "lib"];
#[license = "MIT"];

use std::cast;
use std::ptr;
use std::libc;
use std::os::{MemoryMap,MapReadable,MapWritable};

/**
 * JIT a function dynamically. This will compile the contents (x86 instructions)
 * and return a function that you can call normally.
 *
 * @safe
 * @param {&[u8]} contents
 */

fn jit_func<T>(region: &MemoryMap, contents: &[u8]) -> T {
    unsafe {
        ptr::copy_memory(region.data, contents.as_ptr(), region.len);
        libc::mprotect(region.data as *libc::c_void, 
                       region.len as libc::size_t, 
                       libc::PROT_EXEC | libc::PROT_READ as libc::c_int);
        assert_eq!(*(contents.as_ptr()), *region.data);
        cast::transmute(region.data)
    }
}

#[test]
fn test_jit_func() {

    let contents = [
        0x48, 0x89, 0xf8,       // mov %rdi, %rax
        0x48, 0x83, 0xc0, 0x04, // add $4, %rax
        0xc3                    // ret
    ];

    let region = match MemoryMap::new(contents.len(), &[MapReadable, MapWritable]) {
        Ok(map) => map,
        Err(err) => fail!(err)
    };

    let Add = jit_func::<extern "C" fn(int) -> int>(&region, contents);
    assert_eq!(Add(4), 8);
    println!("value: {}", Add(4));
}

fn main() {
}
