# Framebuffer

## What is Framebuffer?
Large Map of numbers in RAM that GPU/CPU reads to light up the screen.
- It is a contiguous block of memory!
- It is randomly accessible.
- No rows and columns in ram.
- One long line of bytes
- grid is mathematical illusion to create 2D on 1D hardware

## Pixels
- each pixel is usually 4 bytes (BGR + reserved)
- blue, green, red, reserved (4th byte)
- writing to these bytes changes pixel colors on screen immediately

<!--manual pixel write-->
```rs
let pixel_index = 0;
buffer[pixel_index] = 255; // blue
buffer[pixel_index + 1] = 255; // blue
buffer[pixel_index + 2] = 255; // blue
buffer[pixel_index + 3] = 0; // unused/alpha -> brightness
```

## Traversal
- To move down to next row, cannot directly jump to next row with +1, have to skip the entire row / width of the screen.
- `skip distance` is called the `pitch`.
  - total bytes from start of one row to start of next.
  - **[ pitch = screen-width skip distance ]**

## Offset Formula
- To find any pixel(x,y):
```
Index = (y * pitch) + (x * bytes-per-pixel)
```
- translates 2D coordinates -> 1D index

```rs
fn poke_pixel(x: usize, y:usize, color:(u8,u8,u8), buffer: &mut [u8], pitch: usize, bpp: usize){
    let offset = (y * pitch) + (x * bpp);
    // values between 0 -> 255 (256 values in total)
    // white is 255 all, black is 0 all.
    buffer[ pixel_index      ] = 255; // blue
    buffer[ pixel_index + 1  ] = 255; // green
    buffer[ pixel_index + 2  ] = 255; // red
}
```

## Padding as an Edge Case
Somtimes Pitch is wider than Width * BytesPerPixel. This happens when the hardware wants rows to align with specific memory boundaries. It might create code panic on overflow.
-> Always use Pitch for vertical math, never your own calculated width.

## Project : Fullscreen Gradient
- main MATH part : linear interpolation
- x / width gives a decimal between 0 -> 1.
- multiply that by 255 which is scaling.
- like I use * 100 for percent, which is out of 100, scales everything in the range of 0 -> 100, if I use * 255 it scales everything in the range of 0 -> 255
- now it makes sense -> confusing part was not knowing that I must separate the x & width and * 255 and then look at the equation, like I do on paper but on code's one-line I get confused.

Formula :-
```
Color Value = (Coordinate / Maximum [screen]) * 255
```

- r changes as I move right
- g as I move down
- b stays at a constant value (purple base)

```rs
for y in 0..height {
    for x in 0..width {
        // Calculating color based on position
        let r = (x * 255 / width) as u8;
        let g = (y * 255 / height) as u8;
        let b = 150;

        poke_pixel(x, y, (b, g, r), buffer, pitch, bpp);
    }
}
```

```rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use limine::FramebufferRequest;

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

    let buffer = unsafe {
        core::slice::from_raw_parts_mut(fb.addr() as *mut u8, pitch * height)
    };

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

    loop { unsafe { core::arch::asm!("hlt"); } }
}

/// MODULAR: This function is your "Pen". 
/// It translates (x, y) into the linear framebuffer index.
fn poke_pixel(x: usize, y: usize, color: (u8, u8, u8), buffer: &mut [u8], pitch: usize, bpp: usize) {
    let offset = (y * pitch) + (x * bpp);
    
    // Safety check: ensure we don't write outside the slice
    if offset + 2 < buffer.len() {
        buffer[offset]     = color.0; // Blue
        buffer[offset + 1] = color.1; // Green
        buffer[offset + 2] = color.2; // Red
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("hlt"); } }
}
```

## GRADIENT DANCE :-
- need a time variable, but no system clock like std::time::Instant on bare_metal.
- Use a simple `frame counter`

Math -
```
x * 255 / width -> (x + time) % 255
```
- + shifts color gradient to the right
- % 255 ensures when no. hits 256 it wraps back to 0, preventing crash or weird colors.
- wrapping_add(1): In Rust, if a number gets too big (overflows), the program might panic. wrapping_add tells the CPU: "If you hit the max, just start over at 0 quietly."

- The Refresh Rate: In QEMU, this will run as fast as your CPU allows. It might look like a seizure-inducing strobe light!

- `Screen Tearing`: Since we are writing directly to the memory the GPU is currently reading, you might see a "line" where the top half is the new frame and the bottom is the old one. This is why `"Double Buffering"` exists.
