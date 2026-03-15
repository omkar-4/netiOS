# Rust Docs

## #![]
- crate level attribute (affects whole program)

## no_std
- do not link to rust standard library (comes packaged with rust installation, depends on OS, we don't have an OS here)
- without std I still have the `core` library (called as `crate` in rust).

- with std: my code > std > OS + core + alloc > hardware
- without std: my code > core > hardware/firmware

## core crate
- minimal primitives (types, iterators, slices ...)
- no heap
- no OS/system calls

## no_main
- rust normally inserts a runtime like main()
    - runtime sets up stack
    - sets arguments
    - does integration with OS
- In bare metal we have no runtime, do we disable it

## Imports
use _::_::_
- :: is for path
    - outermost package has sub-packages inside which has more sub-packages inside itself and so on
- treat _::_::_ like a folder/sub-folder/file structure

## asm!
- this is a macro for writing `inline` assembly instructions inside rust code
- asm!("something") - executes 'something' command
- x86 instructions

- OUT port_register, value_register
    - dx: port address, al: value
    - register bindings:
        - in('dx') port,
        - in('al') val,
    - meaning:
        - move port > DX register : 16-bit register
        - move val > AL register : lower 8 bits of AX
    - RAX
        - AX (16 bit)
            - AH
            - AL

## panic!
- another macro which panics/throws error when something goes wrong.
- rust calls errors and crashes as "panic"
- to handle panic we write `panic handlers`

### PanicInfo
- contains : location, message, payload about the panic
- in no_std we define handler outselves

## Serial
- serial port COM1
    - this is a hardware port (place from when you put some things in, like USB port for USBs, in same way for "communication" we use "COM" ports, and yes COM means communication)
    - COM1 is the 1st communication port we send out data to
    - we'll use it to log messages, mainly to read crash logs, errors when kernel itself panics and we cannot rely on GUI logger
    - we cannot rely on software to log software crashes when software itself crashes including the logger

- Our program writes to serial port COM1
- Old PCs expose devices via I/O ports
- each port = 16-bit address
- eg. keyboard (0x60), timer (0x40), serial COM1 (0x3F8)
- writing to port 0x3F8 [0x means a hexadecimal no.] sends a byte to serial interface
    - we loop over our string and send each letter byte by byte to the serial, it looks to us like the entire word/sentence appeared but it was sent like a water through a pipe, lil by lil

### (hexadecimal/base-16 to decimal/base-10 conversion math) :-
0x3F8 = 3*16^2 + F*16 + 8
      = 3*256 + 15*16 + 8
      = 1016
COM1 port address = 1016 in decimal (base 10) system.

## Constant
- const : compile time constant.

syntax :

```rs
const constant_name : data_type = constant_value; 
```

## Data types
- u16 : 16-bit unsigned (positive-only) integer
- 

## outb

syntax -
```rs
outb(port,value)
// send a particular byte value to a port
```

## Unsafe
- when rust cannot guarentee safety when touching hardware
- what can happen : crash CPU, corrupt memory, freeze system, dangling pointers
- unsafe {} block makes risk explicit than implicit

## Options
these are hints for compiler optimization.

options(nomem, nostack, preserves_flags)
- nomem: assembly doesn't access memory
- nostack: won't modify stack pointer
- preserves_flags: CPU flags remain unchanged
::> allow better optimization

## &str
- reference to UTF8 string
- rust string: pointer + length

## loop
for byte in s.bytes()
- bytes() converts string into iterator of u8
- eg. "hi" > h = 104, i = 105 in ascii

string > bytes > for each byte > send to serial port > serial hardware transmits it (serial monitor shows it), if piped to terminal - terminal recieves and prints it

## program entry with _start & no mangle
```rs
#[unsafe(no_mangle)]
```
- normally rust renames symbols.
    - eg. _start → _ZN5crate6_start17h123abc
- no_mangle prevents rust from renaming (mangling)
- bootloader expects exact same name as _start so disable it

