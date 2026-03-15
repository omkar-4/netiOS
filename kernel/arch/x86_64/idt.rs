use core::arch::asm;
use core::cell::UnsafeCell;
use core::mem::size_of;

const IDT_ENTRIES: usize = 256;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    segment_selector: u16,
    ist: u8,
    flags: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

const _: () = assert!(size_of::<IdtEntry>() == 16);

impl IdtEntry {
    const fn empty() -> Self {
        Self {
            offset_low: 0,
            segment_selector: 0,
            ist: 0,
            flags: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
struct Idtr {
    limit: u16,
    base: u64,
}

const _: () = assert!(size_of::<Idtr>() == 10);

struct SyncIdt(UnsafeCell<[IdtEntry; IDT_ENTRIES]>);
unsafe impl Sync for SyncIdt {}

static IDT: SyncIdt = SyncIdt(UnsafeCell::new([IdtEntry::empty(); IDT_ENTRIES]));

pub fn init() {
    unsafe {
        let idt_ptr = IDT.0.get() as *const IdtEntry;

        let idtr = Idtr {
            limit: (size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
            base: idt_ptr as u64,
        };

        asm!(
            "lidt [{}]",
            in(reg) &idtr,
            options(readonly, nostack, preserves_flags)
        );
    }
}