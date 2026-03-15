# APIC Verification and Stress Testing Checklist

## 1. Legacy PIC Deactivation
- [ ] Verify I/O port `0xA1` (PIC2 data) receives byte `0xFF` to mask all interrupts.
- [ ] Verify I/O port `0x21` (PIC1 data) receives byte `0xFF` to mask all interrupts.
- [ ] Ensure legacy PIC is disabled *before* any LAPIC registers are accessed.

## 2. Hardware Discovery (MSR `0x1B`)
- [ ] Verify `rdmsr` correctly reads `IA32_APIC_BASE` (register `0x1B`).
- [ ] Verify the hardware Global Enable bit (Bit 11) is checked or set to `1`.
- [ ] Verify the physical base address is correctly masked (Bits 12-51) to strip out status flags.

## 3. Memory Mapping & MMIO Safety
- [ ] Confirm the High Half Direct Map (HHDM) virtual offset provided by the bootloader is added to the physical base address to prevent Page Faults.
- [ ] Verify absolutely no standard variable assignments (`*ptr = value`) are used for MMIO.
- [ ] Verify `core::ptr::write_volatile` is used to write to the APIC registers.
- [ ] Verify `core::ptr::read_volatile` is used to read from the APIC registers.

## 4. Spurious Interrupt Vector Register (SIVR) Setup
- [ ] Confirm the SIVR is addressed at exactly `LAPIC_BASE + 0x0F0`.
- [ ] Verify the value written to SIVR sets Bit 8 (`0x100`) to software-enable the APIC.
- [ ] Verify the Spurious Vector is mapped to the lowest priority ring (e.g., `0xFF`).

## 5. Stress Testing Protocol (Immediate Verification)
- [ ] **Read-back Test:** Use `read_volatile` on the LAPIC ID Register (offset `0x020`) immediately after initialization. If it returns a valid ID (usually `0` for the boot processor) without a Page Fault, the MMIO mapping is perfectly aligned.
- [ ] **Optimization Test:** Compile the kernel in `--release` mode. If the APIC fails to initialize in release mode but works in debug mode, a volatile memory constraint was missed and the compiler optimized away your hardware writes.
