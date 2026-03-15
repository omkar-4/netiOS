# Global Descriptor Table (GDT) Context in x86_64 Long Mode

## Context & Hardware Reality
The GDT is a fundamental x86 memory structure that historically defined where memory "segments" began and ended. In modern 64-bit Long Mode, the CPU enforces a "Flat Memory Model" [web:307, web:311]. This means the hardware entirely ignores the `Base Address` and `Limit` fields for Code and Data segments (treating them as 0 and infinity) [web:307, web:311]. 

However, you **cannot bypass** building the GDT. The x86_64 CPU still relies entirely on the GDT's **Access Byte** to determine:
1. **Privilege Levels:** Are we running Kernel Code (Ring 0) or User Code (Ring 3)?
2. **Execution Mode:** Are we running native 64-bit code (the `L` flag) or legacy 32-bit compatibility code? [web:307]

If Limine drops you into 64-bit mode, it does so using a temporary GDT. If you want to use Interrupts (IDT) or switch to User Space later, you must build and load your own GDT [web:307].

## Structural Rules & Bitmasking (Intel/AMD Architecture)

### 1. The GDT Entries (8 Bytes each)
In 64-bit mode, standard Code and Data descriptors are still 8 bytes long. You must construct them using exact bitwise shifts [web:307].

**Kernel Code Descriptor Breakdown:**
- **Base / Limit:** Ignored, just set to `0x0`.
- **Access Byte (Bits 40-47):** `0x9A`
  - `P` (Present): 1 (Bit 47)
  - `DPL` (Privilege): 00 (Bits 45-46, Ring 0)
  - `S` (System): 1 (Bit 44, Code/Data)
  - `E` (Executable): 1 (Bit 43, Code segment)
  - `DC` (Direction): 0 (Bit 42)
  - `RW` (Readable): 1 (Bit 41)
  - `A` (Accessed): 0 (Bit 40)
- **Flags (Bits 52-55):** `0x2` (or `0x20` shifted)
  - `L` (Long Mode): 1 (Bit 53) - **Critical for 64-bit mode** [web:304]
  - `DB` (Size): 0 (Bit 54) - Must be 0 when L=1 [web:304]

**Required Table Layout (Strict Ordering):**
1. **Index 0 (0x00):** Null Descriptor (Mandatory hardware rule: 8 bytes of zeros).
2. **Index 1 (0x08):** Kernel Code Segment.
3. **Index 2 (0x10):** Kernel Data Segment.
4. **Index 3 (0x18):** User Code Segment (Ring 3).
5. **Index 4 (0x20):** User Data Segment (Ring 3).
6. **Index 5 (0x28):** TSS (Task State Segment) - *Note: In 64-bit mode, a TSS descriptor is 16 bytes long instead of 8!* [web:307]

### 2. The GDT Register (GDTR)
The CPU requires a specific 10-byte pointer structure to find the table [web:311].
- **Limit (16-bit):** The size of the GDT in bytes minus 1.
- **Base (64-bit):** The absolute memory address of the first byte of the GDT.
- *Implementation Constraint:* This struct must use `#[repr(packed)]` in Rust, otherwise the compiler will pad it to 16 bytes for alignment, causing a Triple Fault when `lgdt` reads the wrong memory address.

## Initialization Constraints

Loading the GDT is a two-step process enforced by the CPU:
1. **The `lgdt` Instruction:** You pass the GDTR to the CPU via inline assembly (`lgdt [pointer]`).
2. **The Far Jump (Segment Reload):** `lgdt` does not immediately activate the new Code Segment. You must perform an assembly "Far Jump" to reload the `CS` (Code Segment) register, and manually push the new offsets into the Data Segment registers (`DS`, `SS`, `ES`, `FS`, `GS`).

## AI Implementation Directives

### To-Do
- Use bitwise OR logic (`|`) and shifting (`<<`) to construct descriptors cleanly without massive hexadecimal magic numbers.
- Ensure the GDT array is instantiated as a `static` variable so it lives forever in kernel memory.
- Provide a safe `init_gdt()` wrapper that executes the unsafe `lgdt` and segment reloads.

### Not To-Do
- Do not use `std::vec` to hold the GDT. It must be a fixed-size `[u64; N]` array.
- Do not attempt to map a 32-bit TSS descriptor; x86_64 strictly requires a 128-bit (16-byte) TSS [web:307].
