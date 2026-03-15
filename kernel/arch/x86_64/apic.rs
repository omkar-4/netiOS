use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const APIC_BASE_ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

const PIC1_DATA_PORT: u16 = 0x21;
const PIC2_DATA_PORT: u16 = 0xA1;

const SIVR_OFFSET: u64 = 0x0F0;
const LAPIC_ID_OFFSET: u64 = 0x020;

const SIVR_APIC_ENABLE: u32 = 1 << 8;
const SIVR_SPURIOUS_VECTOR: u32 = 0xFF;

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

unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | (low as u64)
}

fn disable_legacy_pic() {
    unsafe {
        outb(PIC2_DATA_PORT, 0xFF);
        outb(PIC1_DATA_PORT, 0xFF);
    }
}

fn read_lapic_base() -> u64 {
    let msr_value = unsafe { rdmsr(IA32_APIC_BASE_MSR) };
    msr_value & APIC_BASE_ADDRESS_MASK
}

pub fn init(hhdm_offset: u64) -> u32 {
    disable_legacy_pic();

    let lapic_physical_base = read_lapic_base();
    let lapic_virtual_base = lapic_physical_base + hhdm_offset;

    let sivr_address = (lapic_virtual_base + SIVR_OFFSET) as *mut u32;
    let current_sivr = unsafe { read_volatile(sivr_address) };
    unsafe {
        write_volatile(sivr_address, current_sivr | SIVR_APIC_ENABLE | SIVR_SPURIOUS_VECTOR);
    }

    let lapic_id_address = (lapic_virtual_base + LAPIC_ID_OFFSET) as *const u32;
    let lapic_id = unsafe { read_volatile(lapic_id_address) } >> 24;

    lapic_id
}
