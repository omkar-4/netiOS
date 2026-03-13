# Neti OS


## Introduction

NOTE : ***NOT TAKING CONTRIBUTIONS ATP***

Neti is a modern OS development project with self-education about the world of OS as driving mindset.

This is NOT a just-for-fun project. I intend to develop this into a shippable OS product that real users like you can use.

This is also NOT a 'from scratch' project but IS being built from 'first principles'. I started with that mindset but quickly realised how "ancient" of an end project I would end up with, which nobody can practically use today.

So I pivoted from a purist approach to a pragmatist approach - I wanna build a "modern" OS that ("real" "prople") would actually like to try and use.

For the bootloader I have used 'limine', a modern and better 64-bit alternative to popular GRUB for reasons I have mentioned in documentation.

As a bootloader would take me directly in 64-bit mode removing the pain of implementing the journey from my power-button & motherboard hardware to the kernel, so that I can focus on the OS dev and not the hardware-level nuances.

The kernel is NOT the linux kernel. I am building kernel myself. Even the architecture am thinking out on paper, it isn't original but personal - I would pick the best ideas from best minds and wrap them around my own philosophy.

## Progress -
- [ ] Buiilding the Kernel
  - [x] Serial logger
  - [ ] Circular Global Atomic Ring Buffer with PerCpu 

## My Philosophy -
1. Lightweight
2. Battery Efficient
3. Blazing fast, crazy quick response time, no lag, no loading, just get to the real deal.
4. kind-of modular, not so much that my mind can't track what's going on, not so less that my entire codebase becomes a gigantum mess which again my little mind can't track.

## Usage

1. Uncomment these lines in run.sh.
  - You'll need a fork of limine's 'binary' (binary branch is important, we don't need the codebase, just need the binary) OR
  - Run these same lines manually in shell -
    - git clone .. & make -C limine (run this at root, as with -C flag, it changes dir to limine then runs make; else first `cd` into limine cloned folder and just run `make`)

```sh
# git clone https://github.com/limine-bootloader/limine.git --branch=v10.x-binary --depth=1

# make -C limine
```

2. Run the shell script
- if you use bash
```bash
bash run.sh
```

- if you use zsh
```zsh
zsh run.sh
```

### Your Concerns -
- src/          ← kernel code
- limine.conf   ← boot config
- run.sh        ← build script
- Cargo.toml    ← project config

### Not Your Concerns -
- target/       ← compiler output (cargo owns this)
- iso_root/     ← build script creates this temporarily
- netios.iso    ← final build artifact
- limine/       ← third party bootloader

---

THE END
