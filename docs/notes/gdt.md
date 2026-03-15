Your original GDT had exactly 7 slots:

    Null

    Kernel Code

    Kernel Data

    User Code

    User Data

    TSS (Part 1)

    TSS (Part 2)


[task]
objective = "Refactor the GDT implementation to comply with Rust 2024 Edition and x86_64 Long Mode hardware realities."
requirements = [
    "Remove all TSS constants, flags, and array entries. The TSS does not exist in memory yet. Reduce `GDT_ENTRIES` to 5 (Null, KCode, KData, UCode, UData).",
    "Replace `static mut GDT` with `core::cell::SyncUnsafeCell` to comply with Rust 2024 `static_mut_refs` deprecation.",
    "Ensure `SyncUnsafeCell::new` is used for initialization and `GDT.get() as *mut u64` is used for pointer manipulation.",
    "Retain all previous bitmasking logic, GDTR packed struct, and inline assembly for `lgdt` and segment reloading."
]

Because a 64-bit TSS takes up exactly two slots, removing the TSS means we remove two entries. 7 minus 2 leaves 5.

