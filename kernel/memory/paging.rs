use core::arch::asm;
use core::mem::size_of;
use core::ptr::{read_volatile, write_volatile};

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_WRITE_THROUGH: u64 = 1 << 3;
pub const PAGE_CACHE_DISABLE: u64 = 1 << 4;
pub const PAGE_HUGE: u64 = 1 << 7;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;

pub const PHYS_ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

const _: () = assert!(size_of::<PageTable>() == 4096);

impl PageTable {
    pub const fn new() -> Self {
        Self { entries: [0; 512] }
    }
}

pub fn pml4_index(vaddr: u64) -> usize {
    ((vaddr >> 39) & 0x1FF) as usize
}

pub fn pdpt_index(vaddr: u64) -> usize {
    ((vaddr >> 30) & 0x1FF) as usize
}

pub fn pd_index(vaddr: u64) -> usize {
    ((vaddr >> 21) & 0x1FF) as usize
}

pub fn pt_index(vaddr: u64) -> usize {
    ((vaddr >> 12) & 0x1FF) as usize
}

pub fn read_cr3() -> u64 {
    let cr3: u64;
    unsafe {
        asm!(
            "mov {}, cr3",
            out(reg) cr3,
            options(nomem, nostack, preserves_flags)
        );
    }
    cr3
}

pub unsafe fn write_cr3(addr: u64) {
    unsafe {
        asm!(
            "mov cr3, {}",
            in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

pub fn flush_tlb(addr: u64) {
    unsafe {
        asm!(
            "invlpg [{}]",
            in(reg) addr,
            options(nostack, preserves_flags)
        );
    }
}

fn ensure_table_entry(
    table_phys: u64,
    index: usize,
    flags: u64,
    hhdm_offset: u64,
    allocate_frame: &mut dyn FnMut() -> u64,
) -> u64 {
    let table_virt = (table_phys + hhdm_offset) as *mut u64;
    let entry = unsafe { read_volatile(table_virt.add(index)) };
    if entry & PAGE_PRESENT != 0 {
        return entry & PHYS_ADDR_MASK;
    }
    let new_frame = allocate_frame();
    let new_entry = (new_frame & PHYS_ADDR_MASK) | flags | PAGE_PRESENT;
    unsafe {
        write_volatile(table_virt.add(index), new_entry);
    }
    new_frame & PHYS_ADDR_MASK
}

pub fn map_page_4kib(
    pml4_phys: u64,
    vaddr: u64,
    paddr: u64,
    flags: u64,
    hhdm_offset: u64,
    allocate_frame: &mut dyn FnMut() -> u64,
) {
    let table_flags = PAGE_PRESENT | PAGE_WRITABLE;

    let pdpt_phys = ensure_table_entry(
        pml4_phys,
        pml4_index(vaddr),
        table_flags,
        hhdm_offset,
        allocate_frame,
    );

    let pd_phys = ensure_table_entry(
        pdpt_phys,
        pdpt_index(vaddr),
        table_flags,
        hhdm_offset,
        allocate_frame,
    );

    let pt_phys = ensure_table_entry(
        pd_phys,
        pd_index(vaddr),
        table_flags,
        hhdm_offset,
        allocate_frame,
    );

    let pt_virt = (pt_phys + hhdm_offset) as *mut u64;
    let pt_entry = (paddr & PHYS_ADDR_MASK) | flags | PAGE_PRESENT;
    unsafe {
        write_volatile(pt_virt.add(pt_index(vaddr)), pt_entry);
    }
}

pub fn map_page_2mib(
    pml4_phys: u64,
    vaddr: u64,
    paddr: u64,
    flags: u64,
    hhdm_offset: u64,
    allocate_frame: &mut dyn FnMut() -> u64,
) {
    let table_flags = PAGE_PRESENT | PAGE_WRITABLE;

    let pdpt_phys = ensure_table_entry(
        pml4_phys,
        pml4_index(vaddr),
        table_flags,
        hhdm_offset,
        allocate_frame,
    );

    let pd_phys = ensure_table_entry(
        pdpt_phys,
        pdpt_index(vaddr),
        table_flags,
        hhdm_offset,
        allocate_frame,
    );

    let pd_virt = (pd_phys + hhdm_offset) as *mut u64;
    let pd_entry = (paddr & 0x000F_FFFF_FFE0_0000) | flags | PAGE_PRESENT | PAGE_HUGE;
    unsafe {
        write_volatile(pd_virt.add(pd_index(vaddr)), pd_entry);
    }
}
