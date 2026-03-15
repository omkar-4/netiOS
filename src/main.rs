#![no_std]
#![no_main]

mod arch;

use core::arch::asm;
use core::arch::x86_64::__cpuid;
use core::panic::PanicInfo;
use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest, RsdpRequest};

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

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

fn print_hex(val: u64) {
    print_serial("0x");
    for i in (0..16).rev() {
        let digit = ((val >> (i * 4)) & 0xF) as u8;
        let char = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + (digit - 10)
        };
        unsafe {
            outb(COM1, char);
        }
    }
    print_serial("\n");
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    arch::x86_64::gdt::init();
    print_serial("[INFO] GDT initialized and loaded.\n");
    arch::x86_64::idt::init();
    print_serial("[INFO] IDT initialized and loaded.\n");
    print_serial("hello om \n");

    // Get CPU Vendor using CPUID
    print_serial("CPU ID: ");
    let res = __cpuid(0);
    for reg in [res.ebx, res.edx, res.ecx] {
        for i in 0..4 {
            let byte = ((reg >> (i * 8)) & 0xFF) as u8;
            unsafe {
                outb(COM1, byte);
            }
        }
    }
    print_serial("\n");

    // Get RAM details from Limine
    if let Some(mem_response) = MEMORY_MAP_REQUEST.get_response() {
        let entries = mem_response.entries();
        print_serial("Total Memory Regions Detected: ");
        print_hex(entries.len() as u64);
        if entries.len() > 0 {
            print_serial("First region base address: ");
            print_hex(entries[0].base);
        }
    } else {
        print_serial("Failed to get memory map.\n");
    }

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print_serial("PANIC! \n");
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
