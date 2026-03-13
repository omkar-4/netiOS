#![no_std]
#![no_main]

mod cpu;
mod idt;
mod logger;
mod memory;
mod serial;
mod sfi;

use crate::logger::flush;
use core::panic::PanicInfo;
use limine::BaseRevision;
use limine::request::{FramebufferRequest, MemoryMapRequest};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial::init();
    idt::init();

    // Bootloader Agnostic HAL Translator
    let mut abstract_map = memory::BootMemoryMap {
        regions: [memory::MemoryRegion {
            base: 0,
            pages: 0,
            kind: memory::MemoryKind::Other,
        }; 64],
        count: 0,
    };

    if let Some(mem_response) = MEMORY_MAP_REQUEST.get_response() {
        let entries = mem_response.entries();
        crate::println!(
            "Limine detected {} memory regions. Translating to HAL...",
            entries.len()
        );

        for (i, entry) in entries.iter().enumerate().take(64) {
            use limine::memory_map::EntryType;
            let kind = match entry.entry_type {
                EntryType::USABLE => memory::MemoryKind::Usable,
                EntryType::EXECUTABLE_AND_MODULES => memory::MemoryKind::Kernel,
                EntryType::RESERVED => memory::MemoryKind::Reserved,
                _ => memory::MemoryKind::Other,
            };
            abstract_map.regions[i] = memory::MemoryRegion {
                base: entry.base,
                pages: (entry.length / 4096) as usize,
                kind,
            };
            abstract_map.count += 1;
        }
    }

    // Allocators
    let mut phys_alloc = memory::PhysicalAllocator::init(&abstract_map);
    if let Some(frame) = phys_alloc.alloc_frame() {
        crate::println!(
            "Physical Allocator initialized! Handed out frame: {:#X}",
            frame
        );
    }

    // Start virtual apps way up at the 64GB mark (0x10_0000_0000)
    let mut v_alloc = memory::VirtualBumpAllocator::new(0x1000000000);
    let wasm_app_vaddr = v_alloc.reserve_window(4 * 1024 * 1024 * 1024); // Reserve 4GB
    crate::println!(
        "SASOS Virtual Allocator: Reserved 4GB WASM window at {:#X}",
        wasm_app_vaddr
    );

    // SFI Pointer Masking
    let pks_supported = cpu::enable_pks();
    let security_policy = sfi::WasmSecurityPolicy::new(pks_supported, wasm_app_vaddr);

    if security_policy.pks_enabled {
        crate::println!("SOTA Security: Intel/AMD Hardware PKS enabled for Kernel isolation!");
    } else {
        crate::println!(
            "SOTA Security: PKS not supported. Fallback to SFI Pointer Masking activated!"
        );
    }

    // Simulate the WASM app trying to access memory at offset 0x0000_1234
    let dummy_wasm_ptr: u32 = 0x1234;
    let safe_hardware_ptr = security_policy.compile_safe_ptr(dummy_wasm_ptr);
    crate::println!(
        "WASM Security Check: 32-bit ptr {:#X} strictly mapped to 64-bit {:#X}",
        dummy_wasm_ptr,
        safe_hardware_ptr
    );

    // PKS Security
    if cpu::enable_pks() {
        crate::println!("SOTA Security: Intel PKS enabled for Kernel isolation!");
    } else {
        crate::println!("SOTA Security: CPU does not support PKS (Pass `-cpu max` to QEMU).");
    }

    // verify serial works at boot
    serial::panic_force_write("[OK] COM1 serial initialized");
    serial::panic_force_write("\n this is a msg");

    // push text to lockfree buffer
    crate::println!("hello from my side");
    crate::println!("numbers: {}, Hex:{:#X}", 42, 0xABCD);
    crate::println!(
        "Hello from CPU {}, auto-flushed by hardware timer!",
        cpu::apic_id()
    );

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
