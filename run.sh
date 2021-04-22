#!/bin/sh

mkdir -p esp/efi/boot
cp $1 esp/efi/boot/bootx64.efi
qemu-system-x86_64 -nodefaults -vga std -serial stdio -machine q35 -bios ./OVMF.fd -drive format=raw,file=fat:rw:./esp