# ℹ️ Realix `v0.07`
![Status](https://img.shields.io/badge/status-latest-brightgreen)
![License](https://img.shields.io/github/license/NightFox-YT/Realix)
![Architecture](https://img.shields.io/badge/architecture-x86-blue)

✅ This version is officially supported and frequently updated by the author.

## 📌 About
Realix is a **hybrid OS** designed for x86 architecture, written in NASM.
It supports a built-in boot switcher that lets users choose between a 16-bit Real Mode kernel for legacy compatibility and a high-performance 32-bit Protected Mode kernel.

- **OS size:** `≈9 KB`
- **Initial release:** `June 30, 2026`

## ✨ Key Features
- BIOS-based bootloader
- BIOS-API (for kernel16):
   - VGA text mode (80×25) and video mode (320×200, 256 colors)
   - Reads raw sectors from disk (INT 13h)
   - Read-only FAT12 filesystem support (loads second-stage bootloader and files by filename)
   - Detects available system memory (INT 12h, 15h)
- Interactive CPU mode selector (`switcher.asm`)
- Kernel16:
   - Basic command-line interface
   - 🆕 Network interface card (NIC) driver
   - 🆕 Simple calculator (Only positive nums)
- 🆕 Kernel32:
   - 🆕 VGA text mode (80x25)
   - 🆕 Simple shell
   - 🆕 Advanced GDT
   - 🆕 Simple IDT

### ⏳ Upcoming Features (v0.08)
- Kernel32: Advanced IDT (with interrupt's handlers)
- Kernel32: IRQ vectors, PIC remap

### ❌ Current Limitations
- No memory allocator
- No standart executable support
- No write operations FAT12 support
- No advanced networking stack

## 📸 Preview
![Realix Experience](screencast.mp4)

## 📦 Hardware Requirements
- **CPU:** x86 compatible (i386+ recommended)
- **RAM:** 256 KB or more
- **Motherboard:** BIOS-supported

## 📂 Project Structure
```
.
├─ source/
│  ├─ bios-api/
│  │  ├─ disk/
│  │  │  ├─ init.asm
│  │  │  └─ read.asm
│  │  ├─ fat12/
│  │  │  ├─ file_open.asm
│  │  │  └─ init.asm
│  │  ├─ memory/
│  │  │  ├─ high.asm
│  │  │  └─ low.asm
│  │  └─ video/
│  │     └─ vga.asm
│  ├─ bootloader/
│  │  ├─ bootix.asm
│  │  ├─ initrix.asm
│  │  └─ switcher.asm
│  ├─ kernel16/
│  │  ├─ io/
│  │  │  ├─ print_ctrl.asm
│  │  │  ├─ print_reg.asm
│  │  │  └─ print.asm
│  │  ├─ shell/
│  │  │  ├─ cli.asm
│  │  │  ├─ cmd_calc.asm
│  │  │  ├─ cmd_cls.asm
│  │  │  └─ commands.asm
│  │  └─ main.asm
│  ├─ kernel32/
│  │  ├─ src/
│  │  │  ├─ drivers/
│  │  │  │  ├─ keyboard.rs
│  │  │  │  ├─ mod.rs
│  │  │  │  └─ vga.rs
│  │  │  ├─ gdt.rs
│  │  │  ├─ idt.rs
│  │  │  ├─ main.rs
│  │  │  └─ shell.rs
│  │  ├─ linker.ld
│  │  └─ Makefile
│  ├─ network/rtl8139.asm
│  └─ shared/config.asm
├─ build/
│     # Output directory for compiled binaries and .img (gitignored)
├─ Makefile
├─ LICENSE
├─ README.md
└─ screen.png
```

## 🛠️ Quick Start & Build

**Prerequisites**
* Compiler: `nasm` (Assembly), `rustup` / `cargo` (Rust toolchain)
* Disk Tools: `mtools` (FAT12 image formatting), `coreutils` (dd/image creation)
* (Optional) Emulator: `qemu-system-i386`

### Linux & macOS
1. Install prerequisites
  - Ubuntu/Debian example: `sudo apt update && sudo apt install nasm mtools qemu-system-i386 build-essential curl`
  - macOS example (via Homebrew): `brew install nasm mtools qemu`
  - Rust toolchain (All platforms):
     - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
     - `rustup target add i386-unknown-none`
2. Build & Run via the `Makefile`
  - Only build OS image: `make`
  - Full cycle (build & run in QEMU): `make run`

### Windows
> ⚠️ Native Windows builds are no longer supported as of `v0.03`.
> Please use Linux, macOS, or WSL2 (Recommended) instead.

## 🔗 Links & 🙌 Contributing
- **TikTok:** [tiktok.com/@mainfox.tt](https://www.tiktok.com/@mainfox.tt)
- **Discord:** [discord.gg/Realix](https://discord.gg/D7cATZzSAxp)

Contributions of any kind are welcome:

- 🐞 **Report bugs.**
- 💡 **Suggest new features** or improvements.
- 🔧 **Help optimize or refactor code.**

Feel free to open an issue or reach out via TikTok & Discord.