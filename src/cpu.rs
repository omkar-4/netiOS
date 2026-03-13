// LLVM allows us to push rbx to the stack
// and pop it back seamlessly, avoiding the complex named registers entirely.

// This exactly preserves rbx for LLVM
// while extracting the APIC ID cleanly into ebx_val.

use core::arch::asm;

/// Reads the Local APIC ID using the cpuid instruction.
pub fn apic_id() -> usize {
    let mut ebx_val: u32;
    unsafe {
        asm!(
            "push rbx",             // Save LLVM's rbx
            "cpuid",                // Run cpuid (clobbers eax, ebx, ecx, edx)
            "mov {0:e}, ebx",       // Move the lower 32 bits of ebx into our variable
            "pop rbx",              // Restore LLVM's rbx
            out(reg) ebx_val,
            inout("eax") 1 => _,
            out("ecx") _,
            out("edx") _,
            options(nomem, nostack) // Preserves state, no memory side-effects
        );
    }
    (ebx_val >> 24) as usize
}

// Memory Protection Keys for Supervisor Pages (PKS)
pub fn enable_pks() -> bool {
    let mut ecx_val: u32;
    unsafe {
        // CPUID Leaf 7: Check for PKS support (ECX bit 31)
        asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            inout("eax") 7 => _,
            out("ecx") ecx_val,
            out("edx") _,
            options(nomem, nostack)
        );
    }

    if (ecx_val & (1 << 31)) != 0 {
        unsafe {
            // PKS is supported! Enable it by setting CR4 bit 24
            asm!(
                "mov rax, cr4",
                "or rax, (1 << 24)",
                "mov cr4, rax",
                out("rax") _,
                options(nostack)
            );
        }
        true
    } else {
        false // Hardware/Emulator does not support PKS
    }
}
