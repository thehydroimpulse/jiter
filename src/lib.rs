#[crate_id = "jiter#0.0.1"];
#[desc = "Jiter"];
#[crate_type = "bin"];
#[license = "MIT"];
#[allow(dead_code)];

use std::cast;
use region::MappedRegion;

mod raw;
mod region;
mod safe;

/**
 * JIT a function dynamically. This will compile the contents (x86 instructions)
 * and return a function that you can call normally.
 *
 * @safe
 * @param {&[u8]} contents
 */

fn jit_func<T>(region: &MappedRegion, contents: &[u8]) -> T {
    unsafe {
        safe::memcpy(region, contents);
        safe::mprotect(region, contents);
        assert_eq!(*(contents.as_ptr()), *region.addr);
        cast::transmute(region.addr)
    }
}

#[test]
fn test_jit_func() {

    let contents = [
        0x48, 0x89, 0xf8,       // mov %rdi, %rax
        0x48, 0x83, 0xc0, 0x04, // add $4, %rax
        0xc3                    // ret
    ];

    let region = match safe::mmap(contents.len() as u64) {
        Ok(r) => r,
        Err(err) => fail!(err)
    };

    let Add = jit_func::<extern "C" fn(int) -> int>
        (region, contents);

    assert_eq!(Add(4), 8);
    println!("value: {}", Add(4));
}

fn main() {
    let contents = [
        0x48, 0x89, 0xf8,       // mov %rdi, %rax
        0x48, 0x83, 0xc0, 0x04, // add $4, %rax
        0xc3                    // ret
    ];

    let region = match safe::mmap(contents.len() as u64) {
        Ok(r) => r,
        Err(err) => fail!(err)
    };

    type AddFourFn = extern "C" fn(n: int) -> int;
    let Add = jit_func::<AddFourFn>(region, contents);
    println!("Add Return Value: {}", Add(4));
}
