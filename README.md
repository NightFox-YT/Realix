# 📦 Realix v0.02 | 17.08.25 ![Status](https://img.shields.io/badge/status-archive-red) ![License](https://img.shields.io/github/license/NightFox-YT/Realix)
- **Size:** 65 bytes + `AA55` signature
- **Architecture:** x86

## 📌 Description
Realix is a lightweight, simple 16-bit OS designed for x86 architecture, developed from scratch on NASM x86.<br/>

❗ **WARNING:** Support for this version ended on 24.08.25, please upgrade to a newer version.<br/>
(This means that there will be no more commits and changes in the branch.)

## ✨ Features
- ✔️ BIOS-based bootloader
- ✔️ Text output ("Welcome...") | 🆕
- ⏳ Read from disk
- ❌ No filesystem
- ❌ No user input
- ❌ No internet support
- ❌ No sounds

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** ≥512 bytes
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
- source/
  - bootix.asm
  - kernel/
    - print.asm
- LICENSE
- Makefile
- README.md

## 🛠 Build
**Linux:**
  - Use the ready-made solution `Makefile` with command `make`.<br/>

**Windows/MacOS:**
  - Compile source code with `NASM`.
    - `nasm -f bin source/bootix.asm -o build/bootix.bin -i source/kernel`
  - Use the `DD utility`. (MacOS)
    - `dd if=build/bootix.bin of=build/realix.img bs=512 count=1`
  - Use `Rufus`. (Windows)

## 🔗 Links
- **Discord:** https://discord.gg/zMzpWFgXaH

## 🙌 Join Us
**We welcome all contributions!** How to help:
- 🐞 Report bugs
- 💡 Suggest new features
- 🔧 Optimize code