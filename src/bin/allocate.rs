#![cfg(target_os = "linux")]

use std::arch::asm;

fn main() -> ! {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let byte_num = args
        .first()
        .expect("need number of bytes to allocate")
        .parse::<usize>()
        .expect("number of pages must be a non-negative integer number");

    let v = vec![0xa_u8; byte_num];
    let p = v.as_ptr();

    println!("TRACEE: allocated {} bytes at {p:#?}", v.len());

    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            asm!("int3", in("rax") p)
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            asm!("brk #0", in("x0") p)
        }
    }
}
