// Hardware Initialization Code

unsafe fn init_serial() {
    outb(COM1 + 1, 0x00); // Disable all interrupts
    outb(COM1 + 3, 0x80); // Enable DLAB (unlocks baud rate setting)
    outb(COM1 + 0, 0x03); // Set speed to 38400 baud (low byte)
    outb(COM1 + 1, 0x00); // Set speed to 38400 baud (high byte)
    outb(COM1 + 3, 0x03); // Lock DLAB, set 8 bits, no parity
    outb(COM1 + 2, 0xC7); // Enable FIFO queues
    outb(COM1 + 4, 0x0B); // Enable hardware ready signals
}
