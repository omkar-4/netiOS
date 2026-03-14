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

unsafe extern "C" {
    fn timer_interrupt_handler();
    fn page_fault_handler_stub();
}

pub fn init() {
    unsafe {
        let timer_addr = timer_interrupt_handler as *const () as u64;
        IDT[32] = IdtEntry {
            offset_low: timer_addr as u16,
            selector: 0x08,
            ist: 0,
            type_attr: 0x8E,
            offset_mid: (timer_addr >> 16) as u16,
            offset_high: (timer_addr >> 32) as u32,
            zero: 0,
        };

        // register page fault handler (vector 14)
        let pf_addr = page_fault_handler_stub as *const () as u64;
        IDT[14] = IdtEntry {
            offset_low: pf_addr as u16,
            selector: 0x08, // Kernel Code Segment
            ist: 0,
            type_attr: 0x8E, // Present, Interrupt Gate
            offset_mid: (pf_addr >> 16) as u16,
            offset_high: (pf_addr >> 32) as u32,
            zero: 0,
        };

        let idt_desc = IdtDescriptor {
            size: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            offset: core::ptr::addr_of!(IDT) as u64,
        };

        asm!("lidt [{}]", in(reg) &idt_desc, options(nostack));

        crate::serial::outb(0x20, 0x11);
        crate::serial::outb(0xA0, 0x11);
        crate::serial::outb(0x21, 32);
        crate::serial::outb(0xA1, 40);
        crate::serial::outb(0x21, 4);
        crate::serial::outb(0xA1, 2);
        crate::serial::outb(0x21, 0x01);
        crate::serial::outb(0xA1, 0x01);

        crate::serial::outb(0x21, 0xFF);
        crate::serial::outb(0xA1, 0xFF);
    }
}

// global_asm!(
//     ".global timer_interrupt_handler",
//     "timer_interrupt_handler:",
//     "push rax",
//     "push rcx",
//     "push rdx",
//     "push rsi",
//     "push rdi",
//     "push r8",
//     "push r9",
//     "push r10",
//     "push r11",
//     "call handle_timer",
//     "pop r11",
//     "pop r10",
//     "pop r9",
//     "pop r8",
//     "pop rdi",
//     "pop rsi",
//     "pop rdx",
//     "pop rcx",
//     "pop rax",
//     "iretq",
//     ".global page_fault_handler_stub",
//     "page_fault_handler_stub:",
//     // A Page Fault pushes an error code. We must preserve registers.
//     "pop rsi",
//     "push rax",
//     "push rcx",
//     "push rdx",
//     "push rsi",
//     "push rdi",
//     "push r8",
//     "push r9",
//     "push r10",
//     "push r11",
//     // Grab the exact address that caused the fault from CR2
//     "mov rdi, cr2", // Arg 1: The faulting address
//     // Grab the Error Code from the stack (it was pushed before our registers)
//     // "mov rsi, [rsp + 72]", // Arg 2: The error code
//     "call handle_page_fault",
//     "pop r11",
//     "pop r10",
//     "pop r9",
//     "pop r8",
//     "pop rdi",
//     "pop rsi",
//     "pop rdx",
//     "pop rcx",
//     "pop rax",
//     // We MUST pop the 8-byte error code the CPU pushed, otherwise we crash on return!
//     "add rsp, 8",
//     "iretq"
// );
//

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
    "iretq",
    ".global page_fault_handler_stub",
    "page_fault_handler_stub:",
    // The CPU pushed: SS, RSP, RFLAGS, CS, RIP, Error Code.

    // 1. Save ALL scratch registers (exactly matching the timer handler)
    "push rax",
    "push rcx",
    "push rdx",
    "push rsi",
    "push rdi",
    "push r8",
    "push r9",
    "push r10",
    "push r11",
    // 2. Set up arguments for Rust.
    // Arg 1 (rdi): Faulting address from CR2
    "mov rdi, cr2",
    // Arg 2 (rsi): Error Code.
    // It is buried under the 9 registers we just pushed (9 * 8 = 72 bytes).
    "mov rsi, [rsp + 72]",
    // 3. Call the Rust handler
    "call handle_page_fault",
    // 4. Restore ALL scratch registers perfectly in reverse order
    "pop r11",
    "pop r10",
    "pop r9",
    "pop r8",
    "pop rdi",
    "pop rsi",
    "pop rdx",
    "pop rcx",
    "pop rax",
    // 5. Clean up the 8-byte Error Code the CPU pushed, aligning the stack for iretq
    "add rsp, 8",
    // 6. Return seamlessly
    "iretq"
);

#[unsafe(no_mangle)]
extern "C" fn handle_timer() {
    unsafe {
        crate::serial::outb(0x20, 0x20);
    }
}

// #[unsafe(no_mangle)]
// extern "C" fn handle_page_fault(faulting_address: u64, error_code: u64) {
//     // 1. We only want to auto-allocate if it was a missing page (not a permissions violation)
//     // Error Code Bit 0 (Present bit) is 0 if the page was entirely missing.
//     let is_not_present = (error_code & 1) == 0;

//     if is_not_present {
//         crate::serial::panic_force_write(
//             "\n[IDT] Demand Paging triggered! Healing memory map...\n",
//         );

//         unsafe {
//             // 2. Grab our global pointers
//             let pml4 = crate::paging::PAGE_TABLE_ROOT;
//             let allocator = crate::paging::ALLOCATOR_PTR;
//             let hhdm = crate::paging::GLOBAL_HHDM_OFFSET;

//             // 3. Ensure they are fully initialized before using them
//             if !pml4.is_null() && !allocator.is_null() {
//                 // 4. Align the faulting address down to the nearest 4 KiB boundary
//                 let aligned_fault_addr = faulting_address & !0xFFF;

//                 // 5. Ask the allocator for exactly one physical 4 KiB chip
//                 if let Some(new_frame) = (*allocator).alloc_frame() {
//                     // 6. Map the physical chip to the virtual address on the fly
//                     let result = (*pml4).map_memory(
//                         aligned_fault_addr,
//                         new_frame,
//                         crate::paging::PAGE_PRESENT
//                             | crate::paging::PAGE_WRITABLE
//                             | crate::paging::PAGE_USER,
//                         &mut *allocator,
//                         hhdm,
//                     );

//                     if result.is_ok() {
//                         crate::serial::panic_force_write(
//                             "[IDT] Memory healed! Resuming program execution.\n",
//                         );
//                         return; // Return back to assembly stub, which calls iretq and resumes the program!
//                     }
//                 }
//             }
//         }
//     }

//     // -----------

//     // For right now, just intercept it and print it so we know it works without crashing!
//     crate::serial::panic_force_write("\n[IDT] Intercepted a Page Fault! We caught it safely!\n");
//     crate::println!(
//         "CR2 (Address): {:#x}, Error Code: {}",
//         faulting_address,
//         error_code
//     );

//     // We halt here temporarily until we write the actual allocator logic in the next step.
//     loop {
//         unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
//     }
// }

// #[unsafe(no_mangle)]
// extern "C" fn handle_page_fault(faulting_address: u64, error_code: u64) {
//     crate::serial::panic_force_write("\n[DEBUG] Entered Page Fault Handler!\n");

//     let is_not_present = (error_code & 1) == 0;

//     if is_not_present {
//         crate::serial::panic_force_write("[DEBUG] It is a missing page. Trying to heal...\n");

//         unsafe {
//             let pml4 = crate::paging::PAGE_TABLE_ROOT;
//             let allocator = crate::paging::ALLOCATOR_PTR;

//             if pml4.is_null() || allocator.is_null() {
//                 crate::serial::panic_force_write("[DEBUG] ERROR: Pointers are null!\n");
//             } else {
//                 crate::serial::panic_force_write(
//                     "[DEBUG] Pointers are good. Allocating frame...\n",
//                 );

//                 if let Some(new_frame) = (*allocator).alloc_frame() {
//                     crate::serial::panic_force_write(
//                         "[DEBUG] Frame allocated! Mapping memory...\n",
//                     );

//                     let hhdm = crate::paging::GLOBAL_HHDM_OFFSET;
//                     let aligned_fault_addr = faulting_address & !0xFFF;

//                     let result = (*pml4).map_memory(
//                         aligned_fault_addr,
//                         new_frame,
//                         crate::paging::PAGE_PRESENT
//                             | crate::paging::PAGE_WRITABLE
//                             | crate::paging::PAGE_USER,
//                         &mut *allocator,
//                         hhdm,
//                     );

//                     if result.is_ok() {
//                         crate::serial::panic_force_write(
//                             "[DEBUG] Mapped successfully! Flushing TLB...\n",
//                         );

//                         // Tell the CPU to invalidate (flush) the TLB cache for this specific address
//                         core::arch::asm!("invlpg [{}]", in(reg) aligned_fault_addr, options(nostack, preserves_flags));

//                         crate::serial::panic_force_write("[DEBUG] Resuming program execution.\n");
//                         return;
//                     } else {
//                         crate::serial::panic_force_write("[DEBUG] ERROR: map_memory failed!\n");
//                     }
//                 } else {
//                     crate::serial::panic_force_write("[DEBUG] ERROR: alloc_frame returned None!\n");
//                 }
//             }
//         }
//     }

//     crate::serial::panic_force_write("\n[DEBUG] HALTING OS.\n");
//     loop {
//         unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
//     }
// }

// #[unsafe(no_mangle)]
// extern "C" fn handle_page_fault(faulting_address: u64, error_code: u64) {
//     let is_not_present = (error_code & 1) == 0;

//     if is_not_present {
//         unsafe {
//             let allocator = crate::paging::ALLOCATOR_PTR;
//             let hhdm = crate::paging::GLOBAL_HHDM_OFFSET;

//             if !allocator.is_null() {
//                 // LOOPHOLE FIX: Ask the CPU for the true active Page Table!
//                 let mut cr3: u64;
//                 core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));

//                 // CR3 holds the physical address. Mask off the flags and add HHDM to get the virtual pointer.
//                 let active_pml4_phys = cr3 & !0xFFF;
//                 let active_pml4 = (active_pml4_phys + hhdm) as *mut crate::paging::PageTable;

//                 if let Some(new_frame) = (*allocator).alloc_frame() {
//                     crate::serial::panic_force_write("[DEBUG] Allocator gave us physical frame: ");
//                     crate::println!("{:#x}", new_frame);

//                     let aligned_fault_addr = faulting_address & !0xFFF;

//                     let result = (*active_pml4).map_memory(
//                         aligned_fault_addr,
//                         new_frame,
//                         crate::paging::PAGE_PRESENT
//                             | crate::paging::PAGE_WRITABLE
//                             | crate::paging::PAGE_USER,
//                         &mut *allocator,
//                         hhdm,
//                     );

//                     if result.is_ok() {
//                         crate::serial::panic_force_write(
//                             "[DEBUG] Mapped into TRUE active table! Resuming...\n",
//                         );
//                         core::arch::asm!("invlpg [{}]", in(reg) aligned_fault_addr, options(nostack, preserves_flags));
//                         return;
//                     }
//                 }
//             }
//         }
//     }

//     crate::serial::panic_force_write("\n[DEBUG] HALTING OS.\n");
//     loop {
//         unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
//     }
// }
//

// #[unsafe(no_mangle)]
// extern "C" fn handle_page_fault(faulting_address: u64, error_code: u64) {
//     let is_not_present = (error_code & 1) == 0;

//     if is_not_present {
//         unsafe {
//             let allocator = crate::paging::ALLOCATOR_PTR;
//             let hhdm = crate::paging::GLOBAL_HHDM_OFFSET;

//             if !allocator.is_null() {
//                 let mut cr3: u64;
//                 core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));

//                 let active_pml4_phys = cr3 & !0xFFF;
//                 let active_pml4 = (active_pml4_phys + hhdm) as *mut crate::paging::PageTable;

//                 if let Some(new_frame) = (*allocator).alloc_frame() {
//                     let aligned_fault_addr = faulting_address & !0xFFF;

//                     let result = (*active_pml4).map_memory(
//                         aligned_fault_addr,
//                         new_frame,
//                         crate::paging::PAGE_PRESENT
//                             | crate::paging::PAGE_WRITABLE
//                             | crate::paging::PAGE_USER,
//                         &mut *allocator,
//                         hhdm,
//                     );

//                     if result.is_ok() {
//                         crate::serial::panic_force_write("[IDT] Memory healed! Resuming...\n");
//                         core::arch::asm!("invlpg [{}]", in(reg) aligned_fault_addr, options(nostack, preserves_flags));
//                         return; // Successfully return to iretq!
//                     }
//                 }
//             }
//         }
//     }

//     crate::serial::panic_force_write("\n!! UNRECOVERABLE PAGE FAULT !!\n");
//     crate::println!("CR2: {:#x}, Error Code: {}", faulting_address, error_code);
//     loop {
//         unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
//     }
// }

#[unsafe(no_mangle)]
extern "C" fn handle_page_fault(faulting_address: u64, error_code: u64) {
    crate::serial::panic_force_write("\n[IDT] Page Fault Intercepted safely!\n");
    crate::println!("CR2: {:#x}, Error Code: {}", faulting_address, error_code);

    // We halt cleanly here.
    // No more infinite loops until we properly audit the Physical Allocator.
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
    }
}
