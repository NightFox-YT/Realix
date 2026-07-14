# ℹ️ Realix `v0.09`
![Status](https://img.shields.io/badge/status-latest-brightgreen)
![License](https://img.shields.io/github/license/NightFox-YT/Realix)
![Architecture](https://img.shields.io/badge/architecture-x86-blue)

✅ This version is officially supported and frequently updated by the author.

## 📌 About
Realix is a **hybrid OS** for the x86 architecture. The bootloader and 16-bit kernel are written in x86 assembly (NASM), the 32-bit kernel is written in Rust (`no_std`).
It supports a built-in boot switcher that lets you choose between a 16-bit Real Mode kernel for legacy compatibility and a 32-bit Protected Mode kernel for high performance.

- **OS size:** `≈18.1 KB`
- **Initial release:** `July 6, 2026`

## ✨ Key Features
- BIOS-based bootloader
- BIOS-API (for kernel16):
   - VGA text mode (80×25) and video mode (320×200, 256 colors)
   - Reads raw sectors from disk (INT 13h)
   - Read-only FAT12 filesystem support:
     - Loads second-stage bootloader
     - Loads files by filename
   - Detects available system memory (INT 12h, 15h)
- Interactive CPU mode selector (`switcher.asm`)
- Kernel16:
   - Command-line interface with 🆕 history (Up and down arrows)
     - `calc` - Simple calculator (positive integers only)
     - 🆕 `hex`, `ascii`, `fib` - number utilities
     - 🆕 `len`, `upper`, `lower`, `reverse`, `repeat` - text utilities
     - 🆕 `about`, `beep`, `meminfo`
     - `echo [t]`, `clear/cls`, `reboot`, `shutdown`
   - Network interface card (NIC) driver
- Kernel32:
   - VGA text mode (80x25)
   - Basic command-line interface
     - `uptime` - Show uptime in seconds
     - `matrix` - Fun animation
     - `echo [t]`, `clear/cls`, `reboot`, `shutdown`
   - Advanced GDT with TSS
   - Advanced IDT with ISR (Exceptions, IRQ0 - PIT, IRQ1 - Keyboard)
   - PIC remap
   - 🆕 Spurious interrupt (IRQ7/IRQ15) detection
   - 🆕 BSS zeroing at kernel entry

### ⏳ Upcoming Features (v0.10)
- BIOS-API: Disk / FAT12 file reading, `ls` command
- Kernel32: Frame allocator

### ❌ Current Limitations
- No memory allocator
- No standard executable support
- No write operations FAT12 support
- No advanced networking stack

## 📦 Hardware Requirements
- **CPU:** x86 compatible (i386+ recommended)
- **RAM:** 256 KB or more
- **Motherboard:** BIOS-supported

## 📂 Project Structure
```
.
├─ .cargo/
│  ├─ config.toml
│  └─ i386.json
├─ source/
│  ├─ bios-api/
│  │  ├─ disk/
│  │  │  ├─ init.asm
│  │  │  └─ read.asm
│  │  ├─ fat12/
│  │  │  ├─ file_load.asm
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
│  ├─ crypto/arch_entropy.asm
│  ├─ kernel16/
│  │  ├─ io/
│  │  │  ├─ print_ctrl.asm
│  │  │  ├─ print_reg.asm
│  │  │  └─ print.asm
│  │  ├─ shell/
│  │  │  ├─ cli.asm
│  │  │  ├─ cmd_calc.asm
│  │  │  ├─ cmd_cls.asm
│  │  │  ├─ commands.asm
│  │  │  └─ parse.asm
│  │  ├─ main.asm
│  │  └─ Makefile
│  ├─ kernel32/
│  │  ├─ src/
│  │  │  ├─ commands/
│  │  │  │  ├─ matrix.rs
│  │  │  │  └─ mod.rs
│  │  │  ├─ drivers/
│  │  │  │  ├─ keyboard.rs
│  │  │  │  ├─ mod.rs
│  │  │  │  ├─ pit.rs
│  │  │  │  └─ vga.rs
│  │  │  ├─ x86/
│  │  │  │  ├─ gdt.rs
│  │  │  │  ├─ idt.rs
│  │  │  │  ├─ isr.rs
│  │  │  │  ├─ memory.rs
│  │  │  │  ├─ mod.rs
│  │  │  │  └─ pic.rs
│  │  │  ├─ main.rs
│  │  │  ├─ shell.rs
│  │  │  └─ utils.rs
│  │  ├─ linker.ld
│  │  └─ Makefile
│  ├─ network/rtl8139.asm
│  └─ shared/config.asm
├─ build/
│     # Output directory for compiled binaries and .img (gitignored)
├─ .gitignore
├─ Cargo.lock
├─ Cargo.toml
├─ Contributing.md
├─ LICENSE
├─ Makefile
└─ README.md
```

## 🛠️ Quick Start & Build

**Prerequisites**
* Compiler: `nasm` (Assembly), `rustup` + **nightly** toolchain (Rust)
* Disk Tools: `mtools` (FAT12 image formatting via mformat/mcopy)
* Objcopy: `cargo-binutils` (provides `rust-objcopy`)
* (Optional) Emulator: `qemu-system-x86_64`

### Linux & macOS
1. Install base tools
  - Ubuntu/Debian example: `sudo apt update && sudo apt install nasm mtools qemu-system-x86 curl`
  - macOS example (via Homebrew): `brew install nasm mtools qemu`
2. Install the Rust toolchain (all platforms)
   - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   - `rustup toolchain install nightly`
   - `rustup component add rust-src llvm-tools --toolchain nightly`
   - `cargo install cargo-binutils`
3. Build & Run via the `Makefile`
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

See [Contributing.md](Contributing.md) for guidelines, or open an issue / reach out via TikTok & Discord.