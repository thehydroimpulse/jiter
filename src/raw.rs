use std::libc;
extern {

    pub fn mmap(
        addr : *libc::c_char,
        length : libc::size_t,
        prot : libc::c_int,
        flags  : libc::c_int,
        fd   : libc::c_int,
        offset : libc::off_t
        ) -> *u8;

    pub fn munmap(
        addr : *u8,
        length : libc::size_t
        ) -> libc::c_int;

    pub fn mprotect(
        addr: *libc::c_char,
        length: libc::size_t,
        prot: libc::c_int
        ) -> libc::c_int;

    pub fn memcpy(
        dest: *libc::c_void,
        src: *libc::c_void,
        n: libc::size_t
        ) -> *libc::c_void;
}

pub static PROT_NONE   : libc::c_int = 0x0;
pub static PROT_READ   : libc::c_int = 0x1;
pub static PROT_WRITE  : libc::c_int = 0x2;
pub static PROT_EXEC   : libc::c_int = 0x4;

pub static MAP_SHARED  : libc::c_int = 0x1;
pub static MAP_PRIVATE : libc::c_int = 0x2;