extern crate libc;

use std::mem;
use std::ptr;
use std::os::{MemoryMap, MapReadable, MapWritable};

unsafe fn transmute_harder<T, U>(from: T) -> U {
    let mut to: U = mem::uninitialized();
    // copy_memory and forget can't trigger failure, so `from` and `to` won't have destructor run
    // extraneously
    let p = &mut to as *mut U;
    *p = ptr::read(&from as *const _ as *const U);
    ptr::copy_memory(&mut to as *mut U, &from as *const _ as *const U, 1);
    mem::forget(from);
    to
}

/// A function type in the context of a JIT-compiler.
pub struct Function<T> {
    map: Option<MemoryMap>,
    func: Option<T>
}

impl<T> Function<T> {

    /// Allocate a new `Function<T>`. We don't have enough information
    /// to reserve the memory needed or the function itself.
    pub fn new() -> Function<T> {
        Function {
            map: None,
            func: None
        }
    }

}

/// Dynamically compile a function down to machine code. This will compile the
/// contents (x86 instructions) and return a function that you can call normally.
///
/// For security reasons, you're unable to write to the memory mapped region after
/// (or during) this function.
fn jit_func<T>(region: &MemoryMap, contents: &[u8]) -> T {
    unsafe {
        ptr::copy_memory(region.data(), contents.as_ptr(), region.len());
        libc::mprotect(region.data() as *mut libc::c_void, 
                       region.len() as libc::size_t, 
                       libc::PROT_EXEC | libc::PROT_READ as libc::c_int);

        for i in std::iter::range(0, contents.len()) {
            assert_eq!(*(contents.as_ptr().offset(i as int)),
                       *region.data().offset(i as int));
        }

        transmute_harder(region.data())
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
        Err(err) => panic!(err)
    };

    let Add = jit_func::<extern "C" fn(int) -> int>(&region, contents.as_slice());
    assert_eq!(Add(4), 8);
    println!("value: {}", Add(4));
}
