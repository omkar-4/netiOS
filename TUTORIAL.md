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

