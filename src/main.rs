#![no_std]
#![no_main]

mod logger;
mod serial;

use crate::logger::flush;
use core::panic::PanicInfo;
use limine::BaseRevision;
use limine::request::FramebufferRequest;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial::init();

    // verify serial works at boot
    serial::panic_force_write("[OK] COM1 serial initialized");
    serial::panic_force_write("\n this is a msg");

    // push text to lockfree buffer
    crate::println!("hello from my side");
    crate::println!("numbers: {}, Hex:{:#X}", 42, 0xABCD);

    // run consumer manually
    flush();

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            let width = framebuffer.width() as usize;
            let height = framebuffer.height() as usize;
            let pitch = framebuffer.pitch() as usize;
            let bpp = framebuffer.bpp() as usize / 8;
            let buffer = unsafe {
                core::slice::from_raw_parts_mut(framebuffer.addr() as *mut u8, pitch * height)
            };

            for y in 50..150 {
                for x in 50..150 {
                    if x < width && y < height {
                        let offset = y * pitch + x * bpp;
                        buffer[offset] = 255; // Blue
                        buffer[offset + 1] = 255; // Green
                        buffer[offset + 2] = 255; // Red
                    }
                }
            }
        }
    }

    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial::panic_force_write("\n !! KERNEL PANIC !! \n");
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
