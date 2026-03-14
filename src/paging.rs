use core::arch::asm;

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_HUGE: u64 = 1 << 7; // bit flag for 2 MiB chunks

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn set_address(&mut self, physical_addr: u64, flags: u64) {
        self.0 = (physical_addr & 0x000FFFFF_FFFFF000) | flags;
    }

    pub fn is_present(&self) -> bool {
        (self.0 & PAGE_PRESENT) != 0
    }

    pub fn is_huge(&self) -> bool {
        (self.0 & PAGE_HUGE) != 0
    }

    pub fn physical_address(&self) -> u64 {
        self.0 & 0x000FFFFF_FFFFF000
    }
}

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

    pub fn get_index(virtual_addr: u64, level: u8) -> usize {
        let shift = 12 + (level - 1) * 9;
        ((virtual_addr >> shift) & 0x1FF) as usize
    }

    pub fn map_memory(
        &mut self,
        virtual_addr: u64,
        physical_addr: u64,
        flags: u64,
        allocator: &mut crate::memory::PhysicalAllocator,
        hhdm_offset: u64,
    ) -> Result<(), &'static str> {
        let pml4_index = Self::get_index(virtual_addr, 4);
        let pdpt_index = Self::get_index(virtual_addr, 3);
        let pd_index = Self::get_index(virtual_addr, 2);
        let pt_index = Self::get_index(virtual_addr, 1);

        let pml4_entry = &mut self.entries[pml4_index];
        if !pml4_entry.is_present() {
            let new_frame = allocator
                .alloc_frame()
                .ok_or("Out of physical memory for PDPT")?;
            pml4_entry.set_address(new_frame, PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER);
            unsafe {
                core::ptr::write_bytes((new_frame + hhdm_offset) as *mut u8, 0, 4096);
            }
        }
        let pdpt =
            unsafe { &mut *((pml4_entry.physical_address() + hhdm_offset) as *mut PageTable) };

        let pdpt_entry = &mut pdpt.entries[pdpt_index];
        if !pdpt_entry.is_present() {
            let new_frame = allocator
                .alloc_frame()
                .ok_or("Out of physical memory for PD")?;
            pdpt_entry.set_address(new_frame, PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER);
            unsafe {
                core::ptr::write_bytes((new_frame + hhdm_offset) as *mut u8, 0, 4096);
            }
        }
        let pd = unsafe { &mut *((pdpt_entry.physical_address() + hhdm_offset) as *mut PageTable) };

        let pd_entry = &mut pd.entries[pd_index];

        // ADD THIS OVERLAP CHECK:
        if pd_entry.is_present() && pd_entry.is_huge() {
            return Ok(()); // Already covered by a blazing-fast 2 MiB Huge Page
        }

        if !pd_entry.is_present() {
            let new_frame = allocator
                .alloc_frame()
                .ok_or("Out of physical memory for PT")?;
            pd_entry.set_address(new_frame, PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER);
            unsafe {
                core::ptr::write_bytes((new_frame + hhdm_offset) as *mut u8, 0, 4096);
            }
        }
        let pt = unsafe { &mut *((pd_entry.physical_address() + hhdm_offset) as *mut PageTable) };

        let pt_entry = &mut pt.entries[pt_index];
        pt_entry.set_address(physical_addr, flags);

        Ok(())
    }

    pub fn map_memory_2mb(
        &mut self,
        virtual_addr: u64,
        physical_addr: u64,
        flags: u64,
        allocator: &mut crate::memory::PhysicalAllocator,
        hhdm_offset: u64,
    ) -> Result<(), &'static str> {
        let pml4_index = Self::get_index(virtual_addr, 4);
        let pdpt_index = Self::get_index(virtual_addr, 3);
        let pd_index = Self::get_index(virtual_addr, 2);

        let pml4_entry = &mut self.entries[pml4_index];
        if !pml4_entry.is_present() {
            let new_frame = allocator
                .alloc_frame()
                .ok_or("Out of physical memory for PDPT")?;
            pml4_entry.set_address(new_frame, PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER);
            unsafe {
                core::ptr::write_bytes((new_frame + hhdm_offset) as *mut u8, 0, 4096);
            }
        }
        let pdpt =
            unsafe { &mut *((pml4_entry.physical_address() + hhdm_offset) as *mut PageTable) };

        let pdpt_entry = &mut pdpt.entries[pdpt_index];
        if !pdpt_entry.is_present() {
            let new_frame = allocator
                .alloc_frame()
                .ok_or("Out of physical memory for PD")?;
            pdpt_entry.set_address(new_frame, PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER);
            unsafe {
                core::ptr::write_bytes((new_frame + hhdm_offset) as *mut u8, 0, 4096);
            }
        }
        let pd = unsafe { &mut *((pdpt_entry.physical_address() + hhdm_offset) as *mut PageTable) };

        let pd_entry = &mut pd.entries[pd_index];

        // ADD THIS OVERLAP CHECK:
        if pd_entry.is_present() && !pd_entry.is_huge() {
            return Ok(()); // Already mapped as fine-grained 4 KiB pages, don't corrupt it
        }

        // Stop at Level 2 (Page Directory) and apply the PAGE_HUGE flag
        pd_entry.set_address(physical_addr, flags | PAGE_HUGE);

        Ok(())
    }
}

pub unsafe fn load_cr3(pml4_physical_address: u64) {
    unsafe {
        asm!(
            "mov cr3, rax",
            in("rax") pml4_physical_address,
            options(nostack, preserves_flags)
        );
    }
}
