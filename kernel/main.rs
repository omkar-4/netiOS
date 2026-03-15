mod arch;
mod memory;

use core::arch::asm;
use core::arch::x86_64::__cpuid;
use core::panic::PanicInfo;
use limine::BaseRevision;
use limine::request::{
    ExecutableAddressRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest,
};

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

#[used]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

const COM1: u16 = 0x3F8;

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

    let hhdm_offset = HHDM_REQUEST
        .get_response()
        .map(|r| r.offset())
        .unwrap_or(0xFFFF_8000_0000_0000);

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

    if let Some(mem_response) = MEMORY_MAP_REQUEST.get_response() {
        let entries = mem_response.entries();
        print_serial("[INFO] Memory regions: ");
        print_hex(entries.len() as u64);

        let mut regions = [memory::frame::MemoryRegion {
            base: 0,
            length: 0,
            kind: memory::frame::RegionKind::Reserved,
        }; 64];
        let mut region_count = 0;

        for entry in entries.iter() {
            if region_count >= 64 {
                break;
            }
            use limine::memory_map::EntryType;
            let kind = match entry.entry_type {
                EntryType::USABLE => memory::frame::RegionKind::Usable,
                EntryType::BOOTLOADER_RECLAIMABLE => memory::frame::RegionKind::Reclaimable,
                _ => memory::frame::RegionKind::Reserved,
            };
            regions[region_count] = memory::frame::MemoryRegion {
                base: entry.base,
                length: entry.length,
                kind,
            };
            region_count += 1;
        }

        memory::frame::init(&regions[..region_count], hhdm_offset);
        print_serial("[INFO] PMM initialized.\n");
        print_serial("[INFO] Total frames: ");
        print_hex(memory::frame::total_frames() as u64);
        print_serial("[INFO] Free frames: ");
        print_hex(memory::frame::free_frames_count() as u64);

        let frame1 = memory::frame::alloc_frame();
        print_serial("[TEST] alloc_frame #1: ");
        print_hex(frame1);
        assert!(frame1 > 0);
        assert!(frame1 % 4096 == 0);

        let frame2 = memory::frame::alloc_frame();
        print_serial("[TEST] alloc_frame #2: ");
        print_hex(frame2);
        assert!(frame2 > 0);
        assert!(frame2 % 4096 == 0);
        assert!(frame1 != frame2);

        memory::frame::free_frame(frame1);
        print_serial("[TEST] free_frame #1 OK\n");

        let frame3 = memory::frame::alloc_frame();
        print_serial("[TEST] realloc after free: ");
        print_hex(frame3);
        assert!(frame3 == frame1);

        memory::frame::free_frame(frame2);
        memory::frame::free_frame(frame3);
        print_serial("[TEST] All PFA stress tests passed.\n");

        let (kernel_phys_base, kernel_virt_base) =
            if let Some(req) = KERNEL_ADDRESS_REQUEST.get_response() {
                (req.physical_base(), req.virtual_base())
            } else {
                panic!()
            };
        let kernel_virt_offset = kernel_virt_base.wrapping_sub(kernel_phys_base);

        let mut alloc_zeroed = || -> u64 {
            let paddr = memory::frame::alloc_frame();
            unsafe {
                core::ptr::write_bytes((paddr + hhdm_offset) as *mut u8, 0, 4096);
            }
            paddr
        };

        let new_pml4_phys = alloc_zeroed();

        for entry in entries.iter() {
            use limine::memory_map::EntryType;
            let paddr = entry.base;
            let length = entry.length;
            let pages = ((length + 4095) / 4096) as usize;

            if entry.entry_type == EntryType::USABLE
                || entry.entry_type == EntryType::BOOTLOADER_RECLAIMABLE
                || entry.entry_type == EntryType::EXECUTABLE_AND_MODULES
            {
                let flags = memory::paging::PAGE_PRESENT
                    | memory::paging::PAGE_WRITABLE
                    | memory::paging::PAGE_NO_EXECUTE;
                for i in 0..pages {
                    let frame_phys = paddr + (i as u64) * 4096;
                    memory::paging::map_page_4kib(
                        new_pml4_phys,
                        frame_phys + hhdm_offset,
                        frame_phys,
                        flags,
                        hhdm_offset,
                        &mut alloc_zeroed,
                    );
                    memory::paging::map_page_4kib(
                        new_pml4_phys,
                        frame_phys,
                        frame_phys,
                        flags,
                        hhdm_offset,
                        &mut alloc_zeroed,
                    );
                }
            }

            if entry.entry_type == EntryType::EXECUTABLE_AND_MODULES {
                let flags = memory::paging::PAGE_PRESENT | memory::paging::PAGE_WRITABLE;
                for i in 0..pages {
                    let frame_phys = paddr + (i as u64) * 4096;
                    let frame_virt = frame_phys + kernel_virt_offset;
                    memory::paging::map_page_4kib(
                        new_pml4_phys,
                        frame_virt,
                        frame_phys,
                        flags,
                        hhdm_offset,
                        &mut alloc_zeroed,
                    );
                }
            }
        }

        if let Some(fb_response) = FRAMEBUFFER_REQUEST.get_response() {
            if let Some(fb) = fb_response.framebuffers().next() {
                let fb_virt = fb.addr() as u64;
                let fb_phys = fb_virt - hhdm_offset;
                let pitch = fb.pitch() as u64;
                let height = fb.height() as u64;
                let size = pitch * height;

                let align_2mb = 0x20_0000;
                let fb_virt_aligned = fb_virt & !(align_2mb - 1);
                let fb_phys_aligned = fb_phys & !(align_2mb - 1);

                let offset = fb_virt - fb_virt_aligned;
                let aligned_size = (size + offset + align_2mb - 1) & !(align_2mb - 1);
                let pages_2mb = aligned_size / align_2mb;

                let flags = memory::paging::PAGE_PRESENT
                    | memory::paging::PAGE_WRITABLE
                    | memory::paging::PAGE_NO_EXECUTE;
                for i in 0..pages_2mb {
                    let phys = fb_phys_aligned + i * align_2mb;
                    let virt = fb_virt_aligned + i * align_2mb;
                    memory::paging::map_page_2mib(
                        new_pml4_phys,
                        virt,
                        phys,
                        flags,
                        hhdm_offset,
                        &mut alloc_zeroed,
                    );
                }
            }
        }

        let lapic_phys = 0xFEE00000;
        let lapic_virt = lapic_phys + hhdm_offset;
        let lapic_flags = memory::paging::PAGE_PRESENT
            | memory::paging::PAGE_WRITABLE
            | memory::paging::PAGE_CACHE_DISABLE
            | memory::paging::PAGE_WRITE_THROUGH
            | memory::paging::PAGE_NO_EXECUTE;
        memory::paging::map_page_4kib(
            new_pml4_phys,
            lapic_virt,
            lapic_phys,
            lapic_flags,
            hhdm_offset,
            &mut alloc_zeroed,
        );

        unsafe {
            asm!("cli", options(nomem, nostack, preserves_flags));
            memory::paging::write_cr3(new_pml4_phys);
        }

        print_serial("[INFO] CR3 hot-swap successful!\n");

        arch::x86_64::apic::init(hhdm_offset);
        print_serial("[INFO] APIC re-enabled successfully.\n");
    } else {
        print_serial("[ERR] Failed to get memory map.\n");
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
