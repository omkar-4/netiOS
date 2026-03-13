use core::arch::asm;

// x86_64 page table entry flags
pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2; // needed if wasm apps run in ring 3

// single entry in any level of page level
// 8 bytes
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn set_address(&mut self, physical_addr: u64, flags: u64) {
        // Clear all bits, then set the new physical frame address and hardware flags.
        // Physical addresses must be 4KB aligned (lower 12 bits are 0).
        self.0 = (physical_addr & 0x000FFFFF_FFFFF000) | flags;
    }

    pub fn is_present(&self) -> bool {
        (self.0 & PAGE_PRESENT) != 0
    }

    pub fn physical_address(&self) -> u64 {
        self.0 & 0x000FFFFF_FFFFF000
    }
}

// 4KB page table with 512 entries
// repr C align 4096 to guarentee CPU MMU can read it correctly
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::empty(); 512],
        }
    }

    /// Extracts the 9-bit index for a specific table level from a virtual address
    pub fn get_index(virtual_addr: u64, level: u8) -> usize {
        let shift = 12 + (level - 1) * 9;
        ((virtual_addr >> shift) & 0x1FF) as usize
    }
}

// Loads new PML4 (Root Page Table) into CPU's CR3 register
pub unsafe fn load_cr3(pml4_physical_address: u64) {
    unsafe {
        asm!(
            "mov cr3, rax",
            in("rax") pml4_physical_address,
            options(nostack, preserves_flags)
        );
    }
}
