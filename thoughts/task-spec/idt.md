Contextual Documentation: x86_64 IDT (Intel SDM Reference)

Architectural Reality of the 64-bit IDT:
In Long Mode, the CPU handles interrupts by reading a 16-byte Gate Descriptor. Unlike 32-bit mode, the instruction pointer (RIP) is a full 64-bit value, which Intel had to slice into three separate chunks to maintain backwards compatibility with legacy layout structures.

The 16-Byte Gate Structure:

    offset_low (Bits 0-15): Lowest 16 bits of the handler address.

    segment_selector (Bits 16-31): The GDT Kernel Code Selector (Must be 0x08).

    ist (Bits 32-39): Interrupt Stack Table index (Must be 0 until we build the TSS).

    flags (Bits 40-47): Gate Type and Attributes. For a standard kernel interrupt, this is strictly 0x8E (Binary 10001110 -> Present=1, DPL=00, Type=1110).

    offset_mid (Bits 48-63): Middle 16 bits of the handler address.

    offset_high (Bits 64-95): Highest 32 bits of the handler address.

    reserved (Bits 96-127): Must be 0.

Hardware Execution Flow:
When an interrupt fires, the CPU:

    Disables further hardware interrupts.

    Pushes the current SS, RSP, RFLAGS, CS, and RIP to the stack.

    Jumps to the 64-bit address assembled from the three offset fields.

    Expects the handler to return using the iretq instruction.
