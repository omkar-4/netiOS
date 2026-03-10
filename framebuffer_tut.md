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
```rs
//

```
