use core::arch::asm;
use core::cell::UnsafeCell;
use core::mem::size_of;

const GDT_ENTRIES: usize = 5;

const ACCESS_PRESENT: u64 = 1 << 47;
const ACCESS_DPL_RING0: u64 = 0 << 45;
const ACCESS_DPL_RING3: u64 = 3 << 45;
const ACCESS_CODE_DATA: u64 = 1 << 44;
const ACCESS_EXECUTABLE: u64 = 1 << 43;
const ACCESS_READABLE: u64 = 1 << 41;

const FLAG_LONG_MODE: u64 = 1 << 53;

const KERNEL_CODE_ACCESS: u64 =
    ACCESS_PRESENT | ACCESS_DPL_RING0 | ACCESS_CODE_DATA | ACCESS_EXECUTABLE | ACCESS_READABLE;
const KERNEL_CODE_FLAGS: u64 = FLAG_LONG_MODE;

const KERNEL_DATA_ACCESS: u64 =
    ACCESS_PRESENT | ACCESS_DPL_RING0 | ACCESS_CODE_DATA | ACCESS_READABLE;
const KERNEL_DATA_FLAGS: u64 = 0;

const USER_CODE_ACCESS: u64 =
    ACCESS_PRESENT | ACCESS_DPL_RING3 | ACCESS_CODE_DATA | ACCESS_EXECUTABLE | ACCESS_READABLE;
const USER_CODE_FLAGS: u64 = FLAG_LONG_MODE;

const USER_DATA_ACCESS: u64 =
    ACCESS_PRESENT | ACCESS_DPL_RING3 | ACCESS_CODE_DATA | ACCESS_READABLE;
const USER_DATA_FLAGS: u64 = 0;

struct SyncGdt(UnsafeCell<[u64; GDT_ENTRIES]>);
unsafe impl Sync for SyncGdt {}

static GDT: SyncGdt = SyncGdt(UnsafeCell::new([0; GDT_ENTRIES]));

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u64,
}

const _: () = assert!(size_of::<Gdtr>() == 10);

const KERNEL_CODE_SELECTOR: u16 = 0x08;
const KERNEL_DATA_SELECTOR: u16 = 0x10;

fn build_descriptor(access: u64, flags: u64) -> u64 {
    access | flags
}

pub fn init() {
    unsafe {
        let gdt_ptr = GDT.0.get() as *mut u64;
        *gdt_ptr.add(0) = 0;
        *gdt_ptr.add(1) = build_descriptor(KERNEL_CODE_ACCESS, KERNEL_CODE_FLAGS);
        *gdt_ptr.add(2) = build_descriptor(KERNEL_DATA_ACCESS, KERNEL_DATA_FLAGS);
        *gdt_ptr.add(3) = build_descriptor(USER_CODE_ACCESS, USER_CODE_FLAGS);
        *gdt_ptr.add(4) = build_descriptor(USER_DATA_ACCESS, USER_DATA_FLAGS);

        let gdtr = Gdtr {
            limit: (size_of::<[u64; GDT_ENTRIES]>() - 1) as u16,
            base: gdt_ptr as u64,
        };

        asm!(
            "lgdt [{}]",
            in(reg) &gdtr,
            options(readonly, nostack, preserves_flags)
        );

        asm!(
            "push {sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            sel = in(reg) KERNEL_CODE_SELECTOR as u64,
            tmp = lateout(reg) _,
            options(preserves_flags)
        );

        asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov fs, {0:x}",
            "mov gs, {0:x}",
            "mov ss, {0:x}",
            in(reg) KERNEL_DATA_SELECTOR,
            options(nostack, preserves_flags)
        );
    }
}
