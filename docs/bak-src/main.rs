

#![no_std]
#![no_main]

mod cpu;
mod gdt;
mod idt;
mod logger;
mod memory;
mod paging;
mod serial;
mod sfi;
mod tss;

use crate::logger::flush;
use core::arch::asm;
use core::panic::PanicInfo;
use limine::BaseRevision;
use limine::request::{
    ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest,
};

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }

    serial::init();

    tss::init();
    gdt::init();
    idt::init();

    let mut abstract_map = memory::BootMemoryMap {
        regions: [memory::MemoryRegion {
            base: 0,
            pages: 0,
            kind: memory::MemoryKind::Other,
        }; 64],
        count: 0,
    };

    if let Some(mem_response) = MEMORY_MAP_REQUEST.get_response() {
        for (i, entry) in mem_response.entries().iter().enumerate().take(64) {
            use limine::memory_map::EntryType;
            let kind = match entry.entry_type {
                EntryType::USABLE => memory::MemoryKind::Usable,
                EntryType::EXECUTABLE_AND_MODULES => memory::MemoryKind::Kernel,
                EntryType::RESERVED => memory::MemoryKind::Reserved,
                EntryType::BOOTLOADER_RECLAIMABLE => memory::MemoryKind::Bootloader,
                _ => memory::MemoryKind::Other,
            };
            abstract_map.regions[i] = memory::MemoryRegion {
                base: entry.base,
                pages: ((entry.length + 4095) / 4096) as usize,
                kind,
            };
            abstract_map.count += 1;
        }
    };

    let mut phys_alloc = memory::PhysicalAllocator::init(&abstract_map);
    let mut v_alloc = memory::VirtualBumpAllocator::new(0x1000000000);
    let wasm_app_vaddr = v_alloc.reserve_window(4 * 1024 * 1024 * 1024);

    let pks_supported = cpu::enable_pks();
    let security_policy = sfi::WasmSecurityPolicy::new(pks_supported, wasm_app_vaddr);
    let _safe_hardware_ptr = security_policy.compile_safe_ptr(0x1234);

    static mut NEW_PML4: paging::PageTable = paging::PageTable::new();

    let hhdm_offset = HHDM_REQUEST
        .get_response()
        .map(|r| r.offset())
        .unwrap_or(0xFFFF800000000000);

    let (kernel_phys_base, kernel_virt_base) =
        if let Some(req) = KERNEL_ADDRESS_REQUEST.get_response() {
            (req.physical_base(), req.virtual_base())
        } else {
            (0, 0)
        };
    let kernel_virtual_offset = kernel_virt_base.wrapping_sub(kernel_phys_base);

    let legacy_mmio_pages = (2 * 1024 * 1024) / 4096;
    for i in 0..legacy_mmio_pages {
        let phys_addr = i * 4096;
        unsafe {
            let pml4_ptr = core::ptr::addr_of_mut!(NEW_PML4);
            (*pml4_ptr)
                .map_memory(
                    phys_addr,
                    phys_addr,
                    paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                    &mut phys_alloc,
                    hhdm_offset,
                )
                .expect("HW ID");
            (*pml4_ptr)
                .map_memory(
                    phys_addr + hhdm_offset,
                    phys_addr,
                    paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                    &mut phys_alloc,
                    hhdm_offset,
                )
                .expect("HW HHDM");
        }
    }

    // 2MiB blanket pages
    let blanket_pages_2mb = (64 * 1024 * 1024) / 0x200000;
    for i in 0..blanket_pages_2mb {
        let phys_addr = i * 0x200000; // 0x200000 is exactly 2 MiB
        unsafe {
            let pml4_ptr = core::ptr::addr_of_mut!(NEW_PML4);
            (*pml4_ptr)
                .map_memory_2mb(
                    phys_addr,
                    phys_addr,
                    paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                    &mut phys_alloc,
                    hhdm_offset,
                )
                .expect("FB");
            (*pml4_ptr)
                .map_memory_2mb(
                    phys_addr + hhdm_offset,
                    phys_addr,
                    paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                    &mut phys_alloc,
                    hhdm_offset,
                )
                .expect("FB HHDM");
        }
    }

    for i in 0..abstract_map.count {
        let region = &abstract_map.regions[i];
        let mut mapped_bytes: u64 = 0;
        let region_size_bytes = (region.pages as u64) * 4096;

        while mapped_bytes < region_size_bytes {
            let phys_addr = region.base + mapped_bytes;

            unsafe {
                let pml4_ptr = core::ptr::addr_of_mut!(NEW_PML4);

                // Do NOT use 2 MiB pages for the Kernel code. Limine places the kernel at
                // highly specific physical boundaries. Only optimize generic/usable memory.
                if region.kind != memory::MemoryKind::Kernel
                    && phys_addr % 0x200000 == 0
                    && (region_size_bytes - mapped_bytes) >= 0x200000
                {
                    (*pml4_ptr)
                        .map_memory_2mb(
                            phys_addr,
                            phys_addr,
                            paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                            &mut phys_alloc,
                            hhdm_offset,
                        )
                        .expect("FI 2MB");

                    (*pml4_ptr)
                        .map_memory_2mb(
                            phys_addr + hhdm_offset,
                            phys_addr,
                            paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                            &mut phys_alloc,
                            hhdm_offset,
                        )
                        .expect("FH 2MB");

                    mapped_bytes += 0x200000;
                } else {
                    // Fallback to strict 4 KiB mapping for the Kernel and unaligned edges
                    (*pml4_ptr)
                        .map_memory(
                            phys_addr,
                            phys_addr,
                            paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                            &mut phys_alloc,
                            hhdm_offset,
                        )
                        .expect("FI");

                    (*pml4_ptr)
                        .map_memory(
                            phys_addr + hhdm_offset,
                            phys_addr,
                            paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                            &mut phys_alloc,
                            hhdm_offset,
                        )
                        .expect("FH");

                    if region.kind == memory::MemoryKind::Kernel {
                        (*pml4_ptr)
                            .map_memory(
                                phys_addr + kernel_virtual_offset,
                                phys_addr,
                                paging::PAGE_PRESENT | paging::PAGE_WRITABLE, // Executable implicitly via no NX bit yet
                                &mut phys_alloc,
                                hhdm_offset,
                            )
                            .expect("FK");
                    }
                    mapped_bytes += 4096;
                }
            }
        }
    }

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            let fb_virt_base = framebuffer.addr() as u64;
            // The Framebuffer is mapped by Limine in the HHDM, so we find its physical address
            // by subtracting the HHDM offset from its virtual address.
            let fb_phys_base = fb_virt_base - hhdm_offset;
            let fb_size = (framebuffer.pitch() as u64) * (framebuffer.height() as u64);

            // aligned 2MiB
            // align base down and size up
            let align_2mb = 0x200000;
            let fb_virt_aligned = fb_virt_base & !(align_2mb - 1);
            let fb_phys_aligned = fb_phys_base & !(align_2mb - 1);

            let offset_diff = fb_virt_base - fb_virt_aligned;
            let total_size = fb_size + offset_diff;
            let fb_size_aligned = (total_size + align_2mb - 1) & !(align_2mb - 1);

            let mut mapped_bytes = 0;
            while mapped_bytes < fb_size_aligned {
                let phys_addr = fb_phys_aligned + mapped_bytes;
                let virt_addr = fb_virt_aligned + mapped_bytes;
                unsafe {
                    let pml4_ptr = core::ptr::addr_of_mut!(NEW_PML4);
                    (*pml4_ptr)
                        .map_memory_2mb(
                            virt_addr, // Map Limine's Virtual HHDM Pointer
                            phys_addr,
                            paging::PAGE_PRESENT | paging::PAGE_WRITABLE,
                            &mut phys_alloc,
                            hhdm_offset,
                        )
                        .expect("FFB 2MB");
                }
                mapped_bytes += align_2mb;
            }
        }
    }
    unsafe {
        // hand over local variables to global state
        crate::paging::PAGE_TABLE_ROOT = core::ptr::addr_of_mut!(NEW_PML4);
        crate::paging::ALLOCATOR_PTR = core::ptr::addr_of_mut!(phys_alloc);
        crate::paging::GLOBAL_HHDM_OFFSET = hhdm_offset;

        let pml4_virtual_addr = core::ptr::addr_of!(NEW_PML4) as u64;
        let pml4_physical_addr = pml4_virtual_addr - kernel_virtual_offset;

        paging::load_cr3(pml4_physical_addr);
    }

    crate::println!("CR3 Swap Successful! We are running on our own SOTA memory map!");

    serial::panic_force_write("[OK] COM1 serial initialized\n");
    crate::println!("Hello from SOTA SASOS!");
    crate::println!(
        "Hello from CPU {}, auto-flushed by hardware timer!",
        cpu::apic_id()
    );
    flush();

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            let width = framebuffer.width() as usize;
            let height = framebuffer.height() as usize;
            let pitch = framebuffer.pitch() as usize;
            let bpp = framebuffer.bpp() as usize / 8;

            // Because we mapped memory properly, we can safely write to the framebuffer
            // using the virtual address provided by Limine!
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

    // ADD THIS TEST: Deliberately write to an unmapped memory address
    // crate::println!("Attempting to write to unmapped memory to test Demand Paging...");
    // unsafe {
    //     let bad_ptr = 0xDEADC0DE as *mut u8;
    //     *bad_ptr = 42; // This will trigger INT 0x0E (Page Fault)!
    // }

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
