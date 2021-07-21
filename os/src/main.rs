#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]

use core::{fmt::{self, Write}, panic::PanicInfo};

mod lang_items;
mod sbi;

use crate::sbi::sbi_call;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack();
        fn boot_stack_top();
    }
    clear_bss();
    println!("Hello world~~~!");
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    //println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".rodata [{:#x}, --)", srodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(
        "boot_stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    // shotdown();
    panic!("It should shutdown!!!");
}

const SYSCALL_EXIT: usize = 93;
const SYSCALL_WRITE: usize = 64;
const SBI_SHUTDOWN: usize = 8;
const SBI_CONSOLE_PUTCHAR: usize = 1;


fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            :"{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x17}" (id)
            : "memory"
            : "volatile"
        );
    }
    ret
}

pub fn sys_exit(xstate: i32) -> isize {
    syscall(SYSCALL_EXIT, [xstate as usize, 0, 0])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

// #[no_mangle]
// extern "C" fn _start() {
//     println!("Hello world!");
//     sys_exit(9);
// }

pub fn shotdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown");
}

pub fn console_putchar(c: usize) {
    syscall(SBI_CONSOLE_PUTCHAR, [c, 0, 0]);
}

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // sys_write(1, s.as_bytes());
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())       
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

// #[macro_export]
// macro_rules! print {
//     ($fmt: literal $(, $($arg: tt)+)?) => {
//         $crate::console::print(format_args!($fmt $(, $($arg)+)?));
//     };
// }

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("\x1b[31mPanicked at {}:{} {}\x1b[0m", location.file(), location.line(), info.message().unwrap());
    } else {
        println!("\x1b[31mPanicked: {}\x1b[0m", info.message().unwrap());
    }
    shotdown()
}