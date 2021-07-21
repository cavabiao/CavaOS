#!/usr/bin/env bash

echo $(cargo build --release)

file target/riscv64gc-unknown-none-elf/release/os
#rust-objdump -S target/riscv64gc-unknown-none-elf/release/os
rust-readobj -h target/riscv64gc-unknown-none-elf/release/os

rust-objcopy --binary-architecture=riscv64 target/riscv64gc-unknown-none-elf/release/os --strip-all -O binary target/riscv64gc-unknown-none-elf/release/os.bin

qemu-system-riscv64 -machine virt -nographic -bios ../bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000
