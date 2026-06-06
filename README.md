# 🆕 Realix `v0.05`

![Status](https://img.shields.io/badge/status-latest-brightgreen)
![License](https://img.shields.io/github/license/NightFox-YT/Realix)
![Architecture](https://img.shields.io/badge/architecture-x86-blue)
![ASM](https://img.shields.io/badge/assembly-NASM-orange)

✅ This version is officially supported and frequently updated by the author.

## 📌 About
Realix is a **minimal 16-bit OS** designed for x86 architecture, developed from scratch on NASM x86.
- **Size:** `≈2,1 KB`
- **Initial release:** `04.04.25`

## ✨ Features
- ✔️ BIOS-based bootloader
- ✔️ VGA text mode (80×25)
- ✔️ Read from disk
- ✔️ Read the second-stage bootloader with FAT12
- 🆕 VGA video mode (320×200, 256 colors)
- 🆕 Simple beep sound (char)
- 🆕 Collecting Memory information with Loading screen
- 🆕 Load files (kernel) with FAT12
- ⏳ The ability to switch to 32-bit protected mode
- ❌ No user input handling
- ❌ No internet support

## 📦 Hardware Requirements
- **CPU:** x86 (8086+ compatible)
- **RAM:** 1 MB
- **Motherboard:** BIOS-supported

## 📂 File hierarchy
```
.
├─ source/
│  ├─ bootloader/
│  │  ├─ bootix.asm
│  │  └─ initrix.asm
│  ├─ disk/
│  │  ├─ params.asm
│  │  └─ read.asm
│  ├─ drivers/
│  │  ├─ sounds.asm
│  │  └─ vga.asm
│  ├─ fat12/
│  │  ├─ file_open.asm
│  │  └─ init.asm
│  ├─ kernel16/
│  │  ├─ kernel.asm
│  │  ├─ print_reg.asm
│  │  └─ print.asm
│  └─ memory/
│     ├─ get_free.asm
│     ├─ get_lower.asm
│     └─ get_map.asm
├─ build/
├─ Makefile
├─ LICENSE
└─ README.md
```

## 🛠️ Quick Start & Build

**Prerequisites**
* Compiler: `nasm` (Assembly)
* Disk Tools: `mtools` (FAT12 image formatting), `coreutils` (Disk image creation)
* (Optional) Emulator: `qemu-system-i386`

### Linux & macOS
Use the ready-made solution `Makefile`. Simply run: `make`.
(For manual image generation, refer to the `Makefile`)

### Windows
> ⚠️ Windows builds are no longer supported as of `v0.03`.
> Use Linux or macOS instead. WSL2 is also supported.

## 🔗 Links & 🙌 Contributing
- **TikTok:** [tiktok.com/@mainfox.tt](https://www.tiktok.com/@mainfox.tt)
- **Discord:** [discord.gg/Realix](https://discord.gg/D7cATZzSAxp)

Contributions of any kind are welcome:

- 🐞 **Report bugs.**
- 💡 **Suggest new features** or improvements.
- 🔧 **Help optimize or refactor code.**

Feel free to open an issue or reach out via TikTok.