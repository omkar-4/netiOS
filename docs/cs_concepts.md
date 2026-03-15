Memory Address - addresses in the main-RAM (DRAM) address space

CPU Address - can be same as memory address, but its the value CPU places on its address bus. it can be virtual though implemented by paging and gets translated to DRAM address by MMU.

MMU (memory management unit) -  

Pointer
- number that stores an address (usually memory address)
- at hardware level there’s really only one thing: an address stored in a register or memory.
- “Pointer types” are mostly language abstractions.
- Hardware distinctions are mainly by address space:
    - memory pointer (RAM address)
    - I/O port pointer (device address on some CPUs like x86)
    - instruction pointer (PC/IP register holding next instruction address).

Pointer Width
- no. of bits used to store an address


Registers

x86 instructions

I/O ports

Interrupts

Boot Process

Difference between KB and KiB values
- refer to math.md for calculation formulas

Page 