INDEX:
1. Roadmap
2. Intro
3. Building the Kernel

# Roadmap:
- Environment Setup: WSL2, compilers, QEMU emulator ...
- Bootloader
- Kernel
- VGA text driver - driver to print chars and strings on the screen
- CPU Descriptor Tables (GDT) - setup memory segmentation and priveledge levels for the OS
- Interrupt Handling (IDT & ISRs) - config system to catch and manage hardware and software exceptions.
- Programmable Interrupt Controller (PIC) - Remap the hardware chips to properly route external device signals.
- Keyboard Input Driver : Write code to read hardware scan codes and translate them into typed characters.
- Memory Management : Implement a physical and virtual memory allocator to manage RAM.
- File System Integration : Develop a simple system to read and write persistent data on a disk.
- Interactive Shell : Build a command-line interface allowing the user to type and execute basic commands.
- Testing and Debugging : Boot the compiled OS image interactively inside the QEMU emulator.

---

# Introduction

I am using linux OS Ubuntu 22.04 LTS on Windows 11 WSL.
This choice was necessary as OS dev tools and utilities ecosystem largely assumes a linux user at the dev's end.
- formatting raw disk images need tools like (dd) or (mformat)
- to configure bootloaders like GRUB or Limine and manage QEMU eemulation
{these tools are built natively for Linux}
- compiling OS involves writing 1000s of tiny files and building large binary blobs. ext4 linux fs handles it much faster than NTFS. WSL2 >> native win
- Docs for building scripts assume a linux terminal. Powershell translation would be a pain if win chosen.

I started with boot.asm assembly code.
Then I used nasm to turn 'boot.asm' into 'boot.bin' binary executable.

This is how -
```bash
nasm -f bin boot.asm -o boot.bin
```

Then I ran 'boot.bin' using qemu emulator.

This is how -
```bash
qemu-system-x86_64 -fda boot.bin
```

---

Then I started learning about BIOS vs UEFI.
The problem with this code was it would only work in legacy BIOS systems, not on modern hardware.
Till around 2020, for compatibility, motherboards used to have CSM (compatibility support module), a fake simulated BIOS inside UEFI to run old MBR/BIOS code.

For Modern OS dev, I cannot write 16-bit real-mode BIOS asm. I need to write a UEFI app, which is tedious and requires massive framework called EDK II which takes hours to just setup.

So I will use open-source Boot Protocol callled Multiboot2 with an existing UEFI bootloader option called GRUB2 which is normally used in Linux systems already.

By formatting Kernel to be 'Multiboot2 compliant', GRUB2 will handle UEFI hardware handoff, put CPU in 64-bit mode and jump straight into Kernel where OS space begins.

The AI first suggested me to write the Kernel is C, but then I pushed it to do the same in rust. To build with rust, I had to install rust for my WSL, in WSL ubuntu bash terminal.

This is how (this is from rust docs Feb 2026) -
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

But then when I checked it wasn't working and gave error : command not found; I had to source the path into PATH/ENV VARs with {code below} and then when I checked the versions it did show version output.

```bash
source "$HOME/.cargo/env"

rustc --version

cargo --version
```

But always sourcing is a pain, so I can add it to ENV VAR PATH permanently.
This is how -
```bash
echo 'export PATH=$HOME/.cargo/bin:$PATH' >> ~/.bashrc

# reload terminal settings one last time
source ~/.bashrc 
```

Once rust setup is done, I need to install the OS dev specific tools -
```bash
rustup toolchain install nightly

rustup default nightly

rustup component add rust-src llvm-tools-preview

# 1 - standard rust (stable) blocks experimental features. Building OS would need unstable, low-level CPU features like custom memory allocators, that's why the nightly or experimental version of the rust compiler.

# 2 - tells my system to by-default use the nightly version of compiler for all cargo commands from now on.

# 3 - `rust-src` is raw source code of rust std lib. This will be needed to recompile a stripped-down, bare-metal version of it. `llvm-tools-preview` provides low-level binary manipulation tools like 'objcopy' needed to package compiled kernel into bootable image.
```

Then gotta create the Kernel Project.

```bash
cargo new my_custom_os --bin

cd my_custom_os # go inside folder

# If already have a folder
cd '[folder-path]'
cargo init --bin
```

### The Purist vs Pragmatist Approach:

- **Purist:** build everything from scratch, first principles.

- **Pragmatist:** solve problems in a practical, sensible way than having fixed ideas

> **Purist +- :**
- (+) I achieve absolute, god-level understanding of how computer hardware wakes up. There is zero abstraction. I write the 16-bit asm legacy bootloader or raw UEFI app to directly talk to motherboard, read raw sectors of drives, parse fs, manually flip CPU into 64-bit mode etc.

- (-) A bootloader is technically not the Operating System. I might spend months writing drivers just to load a file but won't have written single line of OS code. Bootloader is simply a very complex delivery mechanism. Once my bootloader loads your Kernel into RAM, the bootloader is literally erased from memory and never used again. The control is given to kernel's own hardware drivers that are much more advanced and flexible, but built with constraints over hardware control than UEFI's raw hardware control drivers with major limitations.

> **Pragmatist +- :**
- (+) I immediately start writing OS code. I may use lightweight bootloader like Limine or GRUB to handle undocumented messy motherboard quirks. Bootloader drop me at front door of my OS: inside Kernel, in 64-bit CPU mode, with ready screen to draw pixels on.

- (-) Trust a black box to go from power button to Kernal. I won't learn the insider things.

> 👉🏻 **I will take the Pragmatic Approach**

AI explained me the difference in a great way -

```txt
You might feel that using an existing bootloader like GRUB or Limine means you aren't building an OS "from scratch." This is a common misconception.

Think of building a house. Building the Kernel is like designing the architecture, pouring the foundation, framing the walls, and doing the electrical wiring. Using an existing bootloader is like renting a delivery truck to drop the raw lumber at your construction site. You are still building the entire house from scratch; you just didn't build the delivery truck.
```

# Hardware to Software
1. I press the power button. Power supply unit (PSU) stabilizes electricity and sends a form of 'OK' signal to the motherboard and CPU.
2. CPU wakes up but knows nothing. It blindly looks at hardcoded physical memory address called 'reset vector'. This address points to a flash memory on motherboard containing UEFI (Unified Extensible Firmware Interface) software.
3. UEFI initially runs directly from ROM chip. It turns ON the RAM, comfigures CPU clock and initializes motherboard chipset. Now my PC has working memory (RAM/volatile/temporary memory) at this stage.
4. UEFI scans motherboard for connected hardware and powers them up. GPUs, NVMe SSDs, USB ports, keyboard and mouse - ready to use.
5. UEFI looks at storage drives for small FAT32 file partition called EFI System PArtition (ESP) which has tiny executable code [.efi] which is OS Bootloader.
6. UEFI checks if Bootloader is cyrptograhically signed if 'Secure Boot' is enabled, which IS in modern systems by default. If it passes, UEFI loads bootloader into RAM memory and executes it, now handling major control from motherboard to software code (if I wrote it, then 'my' code).
7. Bootloader will understand disk's file system, find the OS kernel, load it into RAM and jump to it. Kernal will take over from here on.
8. Kernel initializes its own advanced drivers, sets up 'virtual memory', launches 'user programs' like GUI login screen or CLI shell.

## Why does Kernel contains its own hardware drivers?
1. UEFI drivers die at boot: They are designed for pre-boot stage. Once Kernel is loaded to RAM and is ready to take full control, bootloader calls a strict UEFI function `ExitBootServices()` which destroys UEFI driver stack and frees memory they were using so OS can have it.
2. UEFI runs with raw absolute power over hardware. A user program must not be allowed to funnel data through UEFI drivers, where 1 crash in firmware would brick entire system instantly; it also will be vulnerable to security threats. OS drivers run in isolated, protected memory spaces (virtual memory) to prevent this.
3. UEFI aren't built for performance. They use basic polling that constantly/continuously checks for data every defined time-interval that is slow and blocking. Modern OS would use 'interrupt-driven' driver where CPU does other other tasks and hardware just sends an alert when it needs attention.
4. UEFI has no multitasking. An OS juggles hundreds of programs simultaneously. For this, Kernel must write/have complex drivers to queue requests, share hardware resources among multiple apps and virtualize memory - none that UEFI does.

# Building the Kernal:
Now that I have the cargo project-folder all ready - I need to start working on the kernel.

Before the kernel I would need the bootloader, and no am not building that from scratch. Due to reasons above. I will use Limine over GRUB for reasons above.

I will first get binaries of Limine from it's git page 'binary' branch for v10.x.

```bash
git clone https://github.com/limine-bootloader/limine.git --branch=v10.x-binary --depth=1

make -C limine
```

Let me go through the run.sh file now.

Before that, I just wanna say - ISO is gonna come up many times. And I have come across this ISO thing many times before. So I know it is just like a zip file. You combine many files into one file and compress it to reduce size, I mean you don't do it manually, some toold would do it for you - compression and combination, you just have to use that tool.
Here that tool is: xorriso.

```sh
set -e
# -e means exit immediately if a pipeline returns a non-zero status. basically means if an error comes while executing the script then -> exit. stop there.

cargo build
# compiles the rust kernel. binaries are produced at "target/x86_64-unknown-none/debug/netiOS"

mkdir -p ...
# -p creates parent dir(s) if they don't exist

cp ...
# cp target/../netiOS iso_root/../kernel
# it has renamed 'netiOS' to 'kernel'

# cp copies, syntax: cp [options] source(s) destination
# [common options: -r (recursive), -i (interactive, confirm before overwrite), -v (verbose), -f (force copy, removes dest file if it cannot be opened for writing and proceeds with copy), -u (update mode, copies if src is newer than dest or dest is missing), -a (archieve mode; preserve file attr such as perms, ownership, timestamps)]

# we copy 3 limine files: BIOS bootloader (limine-bios.sys) , BIOS CD boot image (limine-bios-cd.bin), UEFI CD boot image (limine-uefi-cd.bin)

# BOOTX64.EFI (UEFI bootloader for 64-bit systems)
# BOOTIA32.EFI (UEFI bootloader for 32-bit systems)
# UEFI firmware looks for .EFI files in /EFI/BOOT/ - standard location

xorriso -as -mkisofs ...
# xorr+iso is a tool that creates ISO files. -as mkisofs (pretend to be mkisofs which is older tool for compatibility)
# -b boot/limine/limine-bios-cd.bin (flag: BIOS boot image location inside ISO)
# -no-emul-boot (Don't emulate floppy disk — use actual boot image directly.)
# -boot-load-size 4 (load 4 sectors of boot image. Legacy BIOS requirement)
# -boot-info-table (inject boot info into boot image, limine needs this)
# --efi-boot boot/limine/limine-uefi-cd.bin (UEFI boot image location inside ISO)
# -efi-boot-part --efi-boot-image (Create a proper UEFI boot partition inside ISO)
# --protective-msdos-label (Add MBR protective label — makes disk tools recognize ISO correctly.)
# iso_root -o netios.iso (iso_root is source folder which is to be packed inside ISO, output file name is netios.iso)
# ./limine/limine bios-install netios.iso (Installs Limine BIOS bootloader directly into ISO's boot sector. BIOS needs this to find bootloader)

qemu-system-x86_64 -cdrom netios.iso -m 512M
# - `qemu-system-x86_64` — emulate x86_64 machine
# - `-cdrom netios.iso` — use your ISO as CD drive
# - `-m 512M` — give VM 512MB RAM, -m means memory (RAM)
```

Some context for myself -
- why bother about floppy disks? (-no-emul-boot) flag - In 1990s CDs couldn't boot directly. BIOS only knew how to boot from floppy disks. They invented a hack - make the CD pretend to be floppy. BIOS thinks it is booting a floppy but actually a CD. the flag says don't pretend the hack, boot directly as a CD which is modern way.

- what is a sector? - storage is divided into "fixed" pieces called sectors. 512 bytes each traditionally. ISO files mimic this structure, not just hardware. ISO format was designed to represent exactly what physical CD looks like, sector by sector. Software ISOs have sectors too. ISO file = virtual CD = same sector structure as real CD

-

Let me walk through the ISO file :-

> netios.iso
- /boot
  - /limine
    - limine-bios-cd.bin
    - limine-uefi.cd.bin
    - limine-bios.sys
  - kernel
- /efi
  - /BOOT
    - BOOTIA32.EFI
    - BOOTX64.EFI
- /[BOOT]
  - 1-Boot-NoEmul.img
  - 2-Boot-NoEmul.img
- boot.catalog
- limine.conf

To install wasmtime and verify installation -
```bash
curl https://wasmtime.dev/install.sh -sSf | bash

wasmtime -V
```

## Building a logger for Kernel:

To build a logger, I must go from changing a pixel color on the screen to printing a sentence. I need:
1. font to define character shapes
2. writer to handle text layout
3. macro to make it easy to use

### Step 1: Font
I need a bitmap font, an image text format for painting the pixels according to character symbols. I'll use PSF1 as AI told me to, where each char is 8*16 grid of bits.
- 1 means "draw a pixel here" (foreground)
- 0 means "skip the pixel" (background)

### Step 2: Character Renderer
- Create the fn that takes a char, looks up its bitmap, loop through bits to call the put_pixel logic.
```rs
fn draw_char(font: &[u8], c: char, x: usize, y: usize) {
    let glyph = &font[c as usize * 16..(c as usize + 1) * 16]; // Get 16 bytes for the char
    for (row_idx, row) in glyph.iter().enumerate() {
        for bit in 0..8 {
            if (row >> (7 - bit)) & 1 == 1 {
                put_pixel(x + bit, y + row_idx, WHITE);
            }
        }
    }
}
```

### Step 3: fmt::Write implementation
To use println!, I need core::gmt::Write trait for global Writer struct. This struct tracks the current (x, y) position and handles "newlines" by resetting x to 0 and increasing y by 16.

With logger I can do -
- Error Handling: Print PANIC: ..something... instead of frozen screen.
- Debugging: Print memory addresses my kernel is using to verify the math.
- Interaction: groundwork for Shell.
