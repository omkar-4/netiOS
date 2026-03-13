use core::arch::{asm, global_asm};

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    type_attr: 0,
    offset_mid: 0,
    offset_high: 0,
    zero: 0,
}; 256];

pub fn init() {
    unsafe {
        // Map Vector 32 (Timer) to our assembly handler
        // cast to raw pointer first then u64
        let timer_addr = timer_interrupt_handler as *const () as u64;
        IDT[32] = IdtEntry {
            offset_low: timer_addr as u16,
            selector: 0x28, // 0x28 is Limine's 64-bit kernel code segment
            ist: 0,
            type_attr: 0x8E, // Present, Ring 0, Interrupt Gate
            offset_mid: (timer_addr >> 16) as u16,
            offset_high: (timer_addr >> 32) as u32,
            zero: 0,
        };

        let idt_desc = IdtDescriptor {
            size: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            offset: core::ptr::addr_of!(IDT) as u64,
        };

        // Load the IDT into the CPU
        asm!("lidt [{}]", in(reg) &idt_desc, options(nostack));

        // Setup the Programmable Interrupt Controller (PIC)
        crate::serial::outb(0x20, 0x11);
        crate::serial::outb(0xA0, 0x11);
        crate::serial::outb(0x21, 32);
        crate::serial::outb(0xA1, 40);
        crate::serial::outb(0x21, 4);
        crate::serial::outb(0xA1, 2);
        crate::serial::outb(0x21, 0x01);
        crate::serial::outb(0xA1, 0x01);

        // Unmask IRQ0 (Timer) on Master PIC, mask everything else
        crate::serial::outb(0x21, 0xFE);
        crate::serial::outb(0xA1, 0xFF);

        // Turn on CPU hardware interrupts!
        asm!("sti", options(nomem, nostack));
    }
}

// Stable Rust way to write a naked assembly interrupt handler.
// We save CPU registers so our Rust code doesn't corrupt running programs.
global_asm!(
    ".global timer_interrupt_handler",
    "timer_interrupt_handler:",
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    "call handle_timer",
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    "iretq"
);

unsafe extern "C" {
    fn timer_interrupt_handler();
}

#[unsafe(no_mangle)]
extern "C" fn handle_timer() {
    // The background task!
    crate::logger::flush();
    // Send EOI to PIC hardware
    unsafe {
        crate::serial::outb(0x20, 0x20);
    }
}
