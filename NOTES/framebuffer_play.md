Right now I have the following code:

<!-- main.rs -->
```rs
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

            for y in 0..height {
                for x in 0..width {
                    if x < width && y < height {
                        let offset = y * pitch + x * bpp;
                        buffer[offset] = 112; // Blue
                        buffer[offset + 1] = 249; // Green
                        buffer[offset + 2] = 33; // Red
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
```

from this entire code above I will just modify one part as shown below.

```rs
for y in 0..height {
    for x in 0..width {
        if x < width && y < height {
            let offset = y * pitch + x * bpp;
            buffer[offset] = 112; // Blue
            buffer[offset + 1] = 249; // Green
            buffer[offset + 2] = 33; // Red
        }
    }
}
```

this creates a rectangle, but now I want a circle.
Mathematically a circle can be represented in graph as x^2 + Y^2 = r^2
I will be using this same equation.

But before let me add for myself some context for changes I have done:
- this loop fills the "entire screen" with a light green color.
  - that is because of 0..height && 0..width
  - I can change that to this code below and it will fill only a small part then, a vertical portrait rectangle.
  ```rs
        for y in 50..120 {
            for x in 50..70 {
  ```
  - i can even target single pixel or very small cluster of pixels and it would be a tiny green dot on the screen
  ```rs
        for y in 100..101 {
            for x in 50..51 {
  ```
  - to make a square:
  ```rs
        for y in 100..200 {
            for x in 100..200 {
  ```

Circle :
- there is this simple formula which is x^2 + y^2 = r^2 for makinga circle on a coordinate plane.
  - but checking every single pixel in a square using multiplication (dx*dx) as in code below is slow. for simple kernel framebuffer it work but for games and high perf needs, we use 'Bresenham's Circle Algorithm' which uses only addition and subtraction.
  - this method creates 'aliasing' as pixels are square (jagged edges, sharp, not-smooth)
  - on non square screens like 1920*1080 cicle will look like an ellipse due to the stretch, then it isn't a circle really though.


```rs
// replace the square for loop with this:
let center_x = 150;
let center_y = 150;
let radius = 50;
let thickness = 2; // How "thick" the stroke is

for y in (center_y - radius - thickness)..(center_y + radius + thickness) {
    for x in (center_x - radius - thickness)..(center_x + radius + thickness) {
        
        // Calculate distance from center: (dx^2 + dy^2)
        let dx = x as i32 - center_x as i32;
        let dy = y as i32 - center_y as i32;
        let distance_sq = (dx * dx) + (dy * dy);
        
        let r_sq = (radius * radius) as i32;
        let outer_r_sq = ((radius + thickness) * (radius + thickness)) as i32;

        // Only draw if the pixel is between the inner and outer edge
        if distance_sq >= r_sq && distance_sq <= outer_r_sq {
            if x < width && y < height {
                let offset = y * pitch + x * bpp;
                buffer[offset] = 255;     // Blue
                buffer[offset + 1] = 0;   // Green
                buffer[offset + 2] = 255; // Red (Makes Magenta)
            }
        }
    }
}
```

now I wanna create a fullscreen pattern of touching circles:
- i will add 2 upper loops to wrap over this circle generation code.

```rs
let radius = 50;
let diameter = radius * 2;
let thickness = 4;
let inner_r = radius - thickness;
let inner_r_sq = (inner_r * inner_r) as i32;

// r_idx is row index and c_idx is col index
// these loops aren't for pixels but for circle counts
for r_idx in 0..(height / diameter) {
    for c_idx in 0..(width / diameter) {
        let center_x = (c_idx * diameter) + radius;
        let center_y = (r_idx * diameter) + radius;
        for y in (center_y - radius)..(center_y + radius) {
            for x in (center_x - radius)..(center_x + radius) {
                // essentially sectioning a square from the screen and then creating an incircle within it

                // dist from center = radius = sqrt(sq.x + sq.y)
                let dx = x as i32 - center_x as i32;
                let dy = y as i32 - center_y as i32;
                let distance_sq = (dx * dx) + (dy * dy);

                let r_sq = (radius * radius) as i32;

                // draw if it is near the edge
                if distance_sq >= inner_r_sq && distance_sq <= r_sq {
                    if x < width && y < height {
                        let offset = (y * pitch) + (x * bpp);
                        buffer[offset] = 179; // blue
                        buffer[offset + 1] = 222; // green
                        buffer[offset + 2] = 245; //red
                    }
                }
            }
        }
    }
}
```

Now to create a honeycomb structure:
- it just shifts the position to create hexagon layout
  - which is the honeycomb structure
```rs
// starting here - same loop
for r_idx in 0..(height / diameter) {
    for c_idx in 0..(width / diameter) {
        let mut center_x = (c_idx * diameter) + radius;
        let center_y = (r_idx * diameter) + radius;

// ------ just add this -------

        // for honeycomb structure
        // won't change shape of circle, just move them to create hexagonal layout
        if r_idx % 2 == 0 {
            center_x += radius;
        }

// -------- till here -----------

        for y in (center_y - radius)..(center_y + radius) {
// ... continue same
```

this modification will let me visualize the formation of a cycle by slowing down the CPU by giving it useless work between pixel lighting/rendering cycles with the delay/timing of 0..100_000 or whatever the number is :

```rs
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

            draw_circle_slowly(
                100,
                (width / 2) as i32,
                (height / 2) as i32,
                buffer,
                pitch,
                bpp,
            );
        }
    }

    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

fn draw_circle_slowly(
    radius: i32,
    center_x: i32,
    center_y: i32,
    buffer: &mut [u8],
    pitch: usize,
    bpp: usize,
) {
    let r_sq = radius * radius;
    let inner_r_sq = (radius - 3) * (radius - 3);

    for y in (center_y - radius)..(center_y + radius) {
        for x in (center_x - radius)..(center_x + radius) {
            let dx = x - center_x;
            let dy = y - center_y;
            let dist_sq = (dx * dx) + (dy * dy);

            if dist_sq >= inner_r_sq && dist_sq <= r_sq {
                let offset = (y as usize * pitch) + (x as usize * bpp);

                // Color it Yellow
                buffer[offset] = 0; // Blue
                buffer[offset + 1] = 255; // Green
                buffer[offset + 2] = 255; // Red

                // Manual delay loop: Adjust this number to change speed
                for _ in 0..100_000 {
                    unsafe {
                        core::ptr::read_volatile(&0);
                    }
                }
            }
        }
    }
}
```

this stupid AI didn't tell me but made a critical change:
- what looked like -
```rs
if x < width && y < height {
let offset = (y * pitch) + (x * bpp);
```
- is now:
```rs
for y in (center_y - radius)..(center_y + radius) {
for x in (center_x - radius)..(center_x + radius) {
```
- which is why `let offset = (y as usize * pitch) + (x as usize * bpp);` type definition -> usize is needed here.
  - In your grid code, x and y always started at 0 and went up. In the new function, the loop starts at a relative point: center_x - radius.
  - It switched to (y as usize * pitch) because your previous code used x and y from a loop that started at 0, making them automatically positive (usize).
  - In the new `draw_circle` function, the loops use `(center_x - radius)`, which can result in negative numbers (`i32`). Since you can't have a negative position in a buffer, it added `as usize` to force the type conversion.
