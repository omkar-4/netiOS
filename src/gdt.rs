use crate::tss::TSS;
use core::arch::asm;

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

// this stays static as CPU reads it constantly
// static mut GDT: [u64; 5] = [
//     0,                  // 0x00: Null descriptor
//     0x00AF9A000000FFFF, // 0x08: Kernel Code (64-bit)
//     0x00CF92000000FFFF, // 0x10: Kernel Data (64-bit)
//     0,                  // 0x18: TSS Descriptor (Low)
//     0,                  // 0x20: TSS Descriptor (High)
// ];

static mut GDT: [u64; 5] = [0; 5];

// static mut GDT_PTR: GdtDescriptor = GdtDescriptor { size: 0, offset: 0 };

pub fn init() {
    unsafe {
        GDT[0] = 0;
        GDT[1] = 0x00AF9A000000FFFF;
        GDT[2] = 0x00CF92000000FFFF;

        // 1. Calculate the TSS descriptor values
        let tss_ptr = core::ptr::addr_of!(TSS) as u64;
        let tss_limit = (core::mem::size_of::<crate::tss::TaskStateSegment>() - 1) as u64;

        let tss_base_low = tss_ptr & 0xFFFFFF;
        let tss_base_middle = (tss_ptr >> 24) & 0xFF;
        let tss_base_high = (tss_ptr >> 32) & 0xFFFFFFFF;

        // 2. Build the lower 8 bytes of the TSS descriptor
        // Flags: Present(1), DPL(00), Type(1001 for 64-bit TSS) -> 0x89
        GDT[3] = (tss_base_middle << 56)
                       | (0x89 << 40) // Access byte
                       | (tss_base_low << 16)
                       | tss_limit;

        // 3. Build the upper 8 bytes of the TSS descriptor
        GDT[4] = tss_base_high;

        // --------------------------------

        let gdt_ptr = GdtDescriptor {
            size: (core::mem::size_of::<[u64; 5]>() - 1) as u16,
            offset: core::ptr::addr_of!(GDT) as u64,
        };

        asm!(
            "lgdt [{0}]",
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
            // Load the Task Register (TR) with offset 0x18 (the TSS)
            "mov ax, 0x18",
            "ltr ax",
            in(reg) &gdt_ptr,
            out("rax") _,
        );
    }
}
