#!/bin/bash
set -e

# git clone https://github.com/limine-bootloader/limine.git --branch=v10.x-binary --depth=1
# make -C limine

echo "Compiling Kernel..."
cargo build

echo "Preparing ISO directory..."
mkdir -p iso_root/boot/limine
mkdir -p iso_root/EFI/BOOT

cp target/x86_64-unknown-none/debug/netiOS iso_root/boot/kernel
cp limine.conf iso_root/limine.conf

cp limine/limine-bios.sys limine/limine-bios-cd.bin limine/limine-uefi-cd.bin iso_root/boot/limine/
cp limine/BOOTX64.EFI iso_root/EFI/BOOT/
cp limine/BOOTIA32.EFI iso_root/EFI/BOOT/

echo "Building ISO..."
xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    iso_root -o netios.iso

./limine/limine bios-install netios.iso

echo "Starting QEMU..."
qemu-system-x86_64 -cdrom netios.iso -m 512M
# optional flags
# `-cpu max`, `-cpu host` with `-enable-kvm`
