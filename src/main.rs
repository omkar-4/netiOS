#![no_std]
#![no_main]

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
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            let width = framebuffer.width() as usize;
            let height = framebuffer.height() as usize;
            let pitch = framebuffer.pitch() as usize;
            let bpp = framebuffer.bpp() as usize / 8;

            // In limine 0.5.0, getting the raw buffer is done via `.addr()`
            // We cast the pointer to a mutable u8 pointer, then create a slice.
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
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
