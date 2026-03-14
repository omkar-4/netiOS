#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const COM1: u16 = 0x3F8;

// Write a byte to the hardware port
unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") val,
            options(nomem, nostack, preserves_flags)
        );
    }
}

fn print_serial(s: &str) {
    for byte in s.bytes() {
        unsafe {
            outb(COM1, byte);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    print_serial("hello om \n");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print_serial("PANIC! \n");
    loop {}
}
