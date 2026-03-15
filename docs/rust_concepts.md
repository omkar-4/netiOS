Reference

Iterators

Attributes

ABI

Ownership



Pointer Types:
• Raw pointers: *const T, *mut T → just memory addresses, unsafe, no checks.
• References: &T, &mut T → safe pointers with borrow rules enforced by the compiler.


## Code to Memory Under the Hood -

```rs
// example code
let x: u8 = 15;
```

1. compilation: rustc turns code into machine code inside executable format like ELF (executable and linkable format)

2. program load: OS loads program > virtual memory > set page table via MMU > now program gets virtual addresses

3. CPU instruction

4. address translation: MMU translates virtual > physical RAM address using page tables (cached in TLB)

5. memory write: CPU sends physical address + value on memory bus

6. DRAM allocation: memory controller selects row + column, activates cell, writes charge to tiny capacitor(s) representing the bit pattern of 15(00001111).

