# IDT Verification Checklist

## 1. Struct Alignment and Memory Layout
- [ ] Verify `IdtEntry` struct is exactly 16 bytes using `const _: () = assert!(core::mem::size_of::<IdtEntry>() == 16);`.
- [ ] Verify `Idtr` pointer struct is exactly 10 bytes using `const _: () = assert!(core::mem::size_of::<Idtr>() == 10);`.
- [ ] Ensure `Idtr` is decorated with `#[repr(C, packed)]` to prevent Rust from inserting padding bytes.

## 2. Bitwise Descriptor Accuracy
- [ ] Verify `offset_low` captures bits 0-15 of the handler address.
- [ ] Verify `offset_mid` captures bits 16-31 of the handler address.
- [ ] Verify `offset_high` captures bits 32-63 of the handler address.
- [ ] Ensure the `segment_selector` is hardcoded to `0x08` (matching the GDT Kernel Code segment).
- [ ] Ensure `flags` is hardcoded to `0x8E` (Interrupt Gate, Present, DPL 0).
- [ ] Ensure `ist` (Interrupt Stack Table) is explicitly set to `0` (TSS is not yet implemented).
- [ ] Verify the final 32 bits (`reserved`) are explicitly set to `0` to prevent CPU #GP faults.

## 3. Global State and Safety
- [ ] Confirm no `static mut` is used for the IDT array.
- [ ] Confirm the stable `UnsafeCell` + `unsafe impl Sync` tuple struct pattern is used for global state.
- [ ] Verify the IDT array is initialized to exactly 256 empty entries: `[IdtEntry::empty(); 256]`.

## 4. Hardware Execution and Loading
- [ ] Verify `lidt` is executed inside an `unsafe` block using `core::arch::asm!`.
- [ ] Ensure the pointer passed to `lidt` is a reference to the 10-byte `Idtr` struct.
- [ ] Ensure no `sti` (Set Interrupt Flag) instruction is executed during initialization (interrupts must remain disabled until handlers are written).
