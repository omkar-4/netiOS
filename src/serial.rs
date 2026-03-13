use core::arch::asm;

const COM1: u16 = 0x3F8;

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!("in al, dx", out("al") ret, in("dx") port, options(nomem, nostack, preserves_flags));
    }
    ret
}

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable all interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(COM1 + 1, 0x00); //                  (hi byte)
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

/// Bypasses all buffers. Instantly clears hardware FIFO and writes directly.
pub fn panic_force_write(s: &str) {
    unsafe {
        outb(COM1 + 2, 0xC7); // Reset/Clear FIFO to destroy any garbled bytes
    }
    for byte in s.bytes() {
        unsafe {
            while (inb(COM1 + 5) & 0x20) == 0 {} // Wait for line idle
            outb(COM1, byte);
        }
    }
}

// write a single byte - used by bg consumer
pub fn write_byte(byte: u8) {
    unsafe {
        while (inb(COM1 + 5) & 0x20) == 0 {
            // wait for line idle
        }
        outb(COM1, byte);
    }
}
