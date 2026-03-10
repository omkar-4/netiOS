# Neti OS
A modern OS development project with self-education about the world of OS as driving mindset.

This is NOT a 'from scratch', 'first principles' based OS. I started with that but quickly realised how "ancient" of an end project I would end up with, which nobody can use practically today.
So I pivoted from a purist approach to a pragmatist approach - I wanna build a "modern" OS that ("real" "prople") would actually like to try and use.

For the bootloader I have used 'limine', a modern and better 64-bit alternative to popular GRUB for reasons I have mentioned in TUTORIAL.md. A bootloader would basically take you to your kernel directly in 64-bit mode removing the pain of travelling from your power-button & motherboard hardware level to the kernel level, so that you can focus on the OS dev and not the hardware-level nuances. Someday maybe, I'll consider that journey separately.

The kernel is NOT the linux kernel. I am building kernel myself. Even the architecture am thinking out on paper, it won't be original, am not that genius ig, but I would pick the best ideas from best minds and wrap them around me own philosophy.

## My Philosophy -
1. lightweight
2. Blazing fast, crazy quick response time, no lag, no loading, just get to the real deal quickest
3. kind-of modular, not so much that my mind can't track what's going on, not so less that my entire code file is a gigantum mess which again my little mind can't track.
4. wake up when its morning, sleep when its night (means use features when they are required for some work, other times put them to sleep and save resources)
5. it works everywhere - coolest part! The native apps' codebase will be portable, the core OS will be portable.

## Usage

1. Uncomment these lines in run.sh as you'll need a fork of limine's 'binary' (binary branch is important, we don't need the codebase, just need the binary) OR run these lines manually in bash shell - git clone .. & make -C limine (run this at root, as with -C, it changes dir to limine then runs make; else first `cd` into limine cloned folder and just run `make`)

```sh
# git clone https://github.com/limine-bootloader/limine.git --branch=v10.x-binary --depth=1

# make -C limine
```

2. run the bash script
```bash
bash run.sh
```

### YOUR CONCERNS -
- src/          ← kernel code
- limine.conf   ← boot config
- run.sh        ← build script
- Cargo.toml    ← project config

### NOT YOUR CONCERNS -
- target/       ← compiler output (cargo owns this)
- iso_root/     ← build script creates this temporarily
- netios.iso    ← final build artifact
- limine/       ← third party bootloader