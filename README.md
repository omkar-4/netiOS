# Neti OS

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
src/          ← kernel code
limine.conf   ← boot config
run.sh        ← build script
Cargo.toml    ← project config

### NOT YOUR CONCERNS -
target/       ← compiler output (cargo owns this)
iso_root/     ← build script creates this temporarily
netios.iso    ← final build artifact
limine/       ← third party bootloader