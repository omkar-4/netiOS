# Page Tables (Virtual Memory) Verification and Stress Testing Checklist

## 1. Struct Alignment and Memory Layout
- [ ] Verify `PageTable` struct is exactly 4096 bytes using `const _: () = assert!(core::mem::size_of::<PageTable>() == 4096);`.
- [ ] Ensure `PageTable` is decorated with `#[repr(align(4096))]` to guarantee it sits on a page boundary in memory.
- [ ] Verify the table contains exactly 512 entries of `u64`.

## 2. Bitwise Flag Accuracy (x86_64 Hardware Rules)
- [ ] Verify `PRESENT` flag is `1 << 0`.
- [ ] Verify `WRITABLE` flag is `1 << 1`.
- [ ] Verify `USER_ACCESSIBLE` flag is `1 << 2`.
- [ ] Verify `HUGE_PAGE` flag is `1 << 7` (Used for 2MB or 1GB pages).
- [ ] Verify `NO_EXECUTE` (NX) flag is `1 << 63`.
- [ ] Verify physical address mask is exactly `0x000FFFFF_FFFFF000` (Clears bits 0-11 and 52-63).

## 3. Virtual Address Parsing
- [ ] Verify PML4 index extracts bits 39-47 `(addr >> 39) & 0x1FF`.
- [ ] Verify PDPT index extracts bits 30-38 `(addr >> 30) & 0x1FF`.
- [ ] Verify PD index extracts bits 21-29 `(addr >> 21) & 0x1FF`.
- [ ] Verify PT index extracts bits 12-20 `(addr >> 12) & 0x1FF`.

## 4. Hardware Execution (CR3 & TLB)
- [ ] Verify `read_cr3()` uses `asm!("mov {}, cr3", out(reg) _)` to get the active PML4 table physical address.
- [ ] Verify `write_cr3()` uses `asm!("mov cr3, {}", in(reg) _)` to swap page tables.
- [ ] Verify `flush_tlb(addr: u64)` uses `asm!("invlpg [{}]", in(reg) addr)` to clear cached translations.

## 5. Stress Testing Protocol (Immediate Verification)
- [ ] **Sanity Read Test:** Call `read_cr3()` and bitwise-AND it with `0x000FFFFF_FFFFF000`. Ensure the lower 12 bits are completely zero. If they are not zero, the address is not page-aligned and the CPU will fault.
- [ ] **Address Parse Test:** Pass a known virtual address (e.g., `0xFFFF_8000_0000_0000`) into the index parser and assert it returns exactly the expected 9-bit indices (e.g., PML4 index `256`).
- [ ] **HHDM Pointer Test:** Take the physical address from `CR3`, add the Bootloader's Higher Half Direct Map (HHDM) virtual offset to it, cast it to a `*const PageTable`, and read index 0. If it triggers a Page Fault, your HHDM offset is incorrect.
