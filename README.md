# Realix v0.04 | 25.08.25 ![Status](https://img.shields.io/badge/status-latest-brightgreen) ![License](https://img.shields.io/github/license/NightFox-YT/Realix)
- **Size:** 617 bytes
- **Architecture:** x86

## 📌 Description
Realix is a lightweight, simple 16-bit OS designed for x86 architecture, developed from scratch on NASM x86.<br/>

✅ This version is officially supported and frequently updated by the author.

## ✨ Features
- ✔️ BIOS-based bootloader
- ✔️ Text output ("Welcome...")
- ✔️ Read from disk
- ✔️ Read files with FAT12 | 🆕
- ❌ No user input
- ❌ No internet support
- ❌ No sounds

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** ≥512 bytes
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
- build/
  - bootix.bin
  - initrix.bin
  - realix.img
- source/
  - bootix.asm
  - initrix.asm
  - disk/
    - lba_to_chs.asm
    - read_file.asm
    - read.asm
  - kernel/
    - print.asm
- LICENSE
- Makefile
- README.md

## 🛠 Build
**Linux:**
  - Use the ready-made solution `Makefile` with command `make`. <br/>

**MacOS:**
  - Compile source code with `NASM`.
    - `nasm -f bin source/bootix.asm -o build/bootix.bin -i source/kernel -i source/disk`
    - `nasm -f bin source/initrix.asm -o build/initrix.bin -i source/kernel`
  - Use the `DD utility`.
    - `dd if=/dev/zero of=build/realix.img bs=512 count=2880`
	- `newfs_msdos -F 12 -f 2880 build/realix.img`
	- `dd if=build/bootix.bin of=build/realix.img conv=notrunc`
  - `mcopy -i build/realix.img build/initrix.bin "::initrix.bin"`

## 🔗 Links
- **Discord:** https://discord.gg/zMzpWFgXaH

## 🙌 Join Us
**We welcome all contributions!** How to help:
- 🐞 Report bugs
- 💡 Suggest new features
- 🔧 Optimize code