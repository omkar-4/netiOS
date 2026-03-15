# GDT Verification & Robustness Checklist

Use this checklist to mathematically and mechanically prove that your custom Global Descriptor Table (GDT) is flawlessly loaded into the x86_64 CPU.

## Phase 1: Boot & Stability 
- [ ] **No Triple Faults:** The kernel boots past the `init_gdt()` function without causing QEMU to infinitely reboot.
- [ ] **Data Segments Reloaded Safely:** Executing `mov ds, ax` (with `0x10` for Kernel Data) succeeds without throwing a General Protection Fault.
- [ ] **Stack Segment Safe:** The `SS` register is successfully reloaded, and subsequent `push`/`pop` operations do not crash the system.

## Phase 2: QEMU Monitor (`info registers`)
*Drop into the QEMU Monitor (press `Ctrl-A`, then `C` in terminal, or use `-monitor stdio`) and run `info registers`.*

- [ ] **GDTR Base Address Match:** The address listed next to `GDT=` perfectly matches the memory address of your static GDT array in Rust. *(Verify against Limine's higher-half memory map, usually starting with `0xFFFFFFFF...`)*.
- [ ] **GDTR Limit Match:** The limit next to `GDT=` equals the exact size of your table minus 1. *(e.g., if you have 6 entries of 8 bytes (48 bytes total) + 1 TSS of 16 bytes = 64 bytes. Limit should be `0x003F`)*.
- [ ] **Code Segment Selector (`CS`):** The `CS` register reads exactly `0008` (Index 1 of your GDT).
- [ ] **Privilege Level (`DPL`):** The `CS` readout confirms `DPL=0` (Kernel Mode).
- [ ] **Long Mode Active:** The `CS` readout explicitly shows the `CS64` flag (proving the `L` bit in the Access Byte is correctly set to `1`).

## Phase 3: Structural Integrity (Static Assertions)
- [ ] **Pointer Packing:** The `Gdtr` struct pointer passed to `lgdt` is strictly verified to be exactly 10 bytes long (using `core::mem::size_of::<Gdtr>() == 10`). *If it is 16 bytes, `#[repr(packed)]` failed.*
- [ ] **Null Descriptor:** Index 0 of the GDT array is proven to be exactly 64 bits of `0x0`.
- [ ] **TSS Size Compliance:** If a Task State Segment (TSS) descriptor is present, it is mapped as a 16-byte descriptor, not an 8-byte descriptor.

## Phase 4: The Ring 3 Trap Test (Optional but recommended)
- [ ] **User Mode Rejection:** Attempting to manually load the `CS` register with a User Code Segment selector (e.g., `0x18 | 3`) from within Kernel Mode correctly triggers a hardware fault, proving the CPU is enforcing the `DPL` ring permissions you mapped. 
