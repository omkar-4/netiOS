#![no_std]
#![no_main]

use core::panic::PanicInfo;
use limine::request::FramebufferRequest;

// The Request: Limine looks for this to initialize the screen
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let response = FRAMEBUFFER_REQUEST.get_response().unwrap();
    let fb = response.framebuffers().next().unwrap();

    let width = fb.width() as usize;
    let height = fb.height() as usize;
    let pitch = fb.pitch() as usize;
    let bpp = (fb.bpp() / 8) as usize;

    let buffer = unsafe { core::slice::from_raw_parts_mut(fb.addr() as *mut u8, pitch * height) };

    // Hands-on: A Gradient Fill
    for y in 0..height {
        for x in 0..width {
            // Calculating color based on position
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 150;

            poke_pixel(x, y, (b, g, r), buffer, pitch, bpp);
        }
    }

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// MODULAR: This function is your "Pen".
/// It translates (x, y) into the linear framebuffer index.
fn poke_pixel(
    x: usize,
    y: usize,
    color: (u8, u8, u8),
    buffer: &mut [u8],
    pitch: usize,
    bpp: usize,
) {
    let offset = (y * pitch) + (x * bpp);

    // Safety check: ensure we don't write outside the slice
    if offset + 2 < buffer.len() {
        buffer[offset] = color.0; // Blue
        buffer[offset + 1] = color.1; // Green
        buffer[offset + 2] = color.2; // Red
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
