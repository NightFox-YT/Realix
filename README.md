# ℹ️ Realix `v0.11`
![Status](https://img.shields.io/badge/status-latest-brightgreen)
![License](https://img.shields.io/github/license/NightFox-YT/Realix)
![Architecture](https://img.shields.io/badge/architecture-x86-blue)

✅ This version is officially supported and frequently updated by the author.

## 📌 About
Realix is a **hybrid OS** for the x86 architecture. The bootloader and 16-bit kernel are written in x86 assembly (NASM), the 32-bit kernel is written in Rust (`no_std`).
It supports a built-in boot switcher that lets you choose between a 16-bit Real Mode kernel for legacy compatibility and a 32-bit Protected Mode kernel for high performance.

- **OS size:** `≈36.5 KB`
- **Initial release:** `August 10, 2026`

## ✨ Key Features
- BIOS-based bootloader: Bootix (stage 1, 512 bytes) + Initrix (stage 2) with 🆕 Auto-boot into the 32-bit kernel after a timeout
- BIOS-API (for kernel16):
   - VGA text mode (`80×25`) and video mode (`320×200, 256 colors`)
   - Reads raw sectors from disk (`INT 13h`) with retries and controller reset
   - Read-only FAT12 filesystem support:
     - Loads second-stage bootloader
     - Loads files by filename
     - Protected memory regions: a file can never overwrite the IVT, bootloader or running kernel
   - Detects available system memory (`int 12h, int 15h E820`)
- Interactive CPU mode selector (`switcher.asm`)
- Kernel16:
   - Command-line interface with history (Up and down arrows)
   - 🆕 16-bit syscall interface (`int 0x80`) with an RLX executable loader — user-mode 16-bit apps live under apps/ and run via `exec`
   - Commands (grouped by category):
     - base: `help`, `clear/cls`, `echo [t]`, `about`, `beep`, `meminfo`, 🆕 `uptime`, 🆕 `sysinfo`
     - numbers: `calc` (positive integers only), `hex`, `fib`
     - text: `len`, `upper`, `lower`, `reverse`, `repeat`, `ascii`
     - system: 🆕 `regs`, 🆕 `time`, 🆕 `date`, 🆕 `vga`, 🆕 `panic`
     - power: `reboot`, `shutdown`
     - fat12: `ls`, `load`, `type` (Print a file as text), `hexdump`, 🆕 `exec` (Run a 16-bit rlx app)
   - 🆕 Panic and exception handlers with a diagnostic screen
   - Network interface card (NIC) driver
- Kernel32:
   - VGA text mode (80×25)
   - Command-line interface
     - `help`, `echo [t]`, `clear/cls`
     - `uptime` - Show uptime in seconds
     - 🆕 `meminfo` - Show memory information
     - `matrix` - Fun animation
     - `reboot`, `shutdown`
   - Advanced GDT with TSS
   - Advanced IDT with ISR (Exceptions, IRQ0 - PIT, IRQ1 - Keyboard)
   - PIC remap, spurious interrupt (IRQ7/IRQ15) detection
   - Frame allocator built from the E820 memory map passed by the bootloader
   - 🆕 NovaAI experiment: run a local Qwen 2.5 0.5B GGUF model from inside kernel32 via the new run-nova / run-nova-nokvm Make targets

### ⏳ Upcoming Features (v0.12)
- Kernel32: 32-bit RLX executable support (currently 16-bit apps only)
- Kernel32: VMM module
- Add new docs for kernel16 & kernel32.

### ❌ Current Limitations
- No dynamic memory allocator
- No 32-bit executable support (RLX loader is 16-bit only)
- No write operations FAT12 support
- No advanced networking stack
- Disk access is CHS-only (no `int 13h` extensions)

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
├─ apps/
│  └─ hello16.asm
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
│  │  └─ vga.asm
│  ├─ bootloader/
│  │  ├─ bootix.asm
│  │  ├─ initrix.asm
│  │  ├─ switcher.asm
│  │  └─ Makefile
│  ├─ crypto/arch_entropy.asm
│  ├─ kernel16/
│  │  ├─ commands/
│  │  │  ├─ base/
│  │  │  │  ├─ cls.asm
│  │  │  │  └─ help.asm
│  │  │  ├─ debug/
│  │  │  │  ├─ key.asm
│  │  │  │  ├─ regs.asm
│  │  │  │  ├─ sysinfo.asm
│  │  │  │  └─ uptime.asm
│  │  │  ├─ file-system/
│  │  │  │  ├─ exec.asm
│  │  │  │  ├─ hexdump.asm
│  │  │  │  ├─ load.asm
│  │  │  │  ├─ ls.asm
│  │  │  │  └─ type.asm
│  │  │  ├─ calc.asm
│  │  │  ├─ nums.asm
│  │  │  ├─ rtc.asm
│  │  │  ├─ text.asm
│  │  │  └─ vga.asm
│  │  ├─ debug/
│  │  │  └─ panic.asm
│  │  ├─ io/
│  │  │  ├─ print.asm
│  │  │  ├─ print_ctrl.asm
│  │  │  └─ print_reg.asm
│  │  ├─ shell/
│  │  │  ├─ cli.asm
│  │  │  ├─ commands.asm
│  │  │  └─ parse.asm
│  │  ├─ main.asm
│  │  ├─ syscall.asm
│  │  └─ Makefile
│  ├─ kernel32/
│  │  ├─ nova-models/
│  │  │  └─ README.md
│  │  ├─ src/
│  │  │  ├─ commands/
│  │  │  │  ├─ echo.rs
│  │  │  │  ├─ help.rs
│  │  │  │  ├─ matrix.rs
│  │  │  │  ├─ meminfo.rs
│  │  │  │  ├─ mod.rs
│  │  │  │  ├─ nova_ai.rs
│  │  │  │  ├─ reboot.rs
│  │  │  │  └─ shutdown.rs
│  │  │  ├─ drivers/
│  │  │  │  ├─ keyboard.rs
│  │  │  │  ├─ mod.rs
│  │  │  │  ├─ pit.rs
│  │  │  │  └─ vga.rs
│  │  │  ├─ memory/
│  │  │  │  ├─ frame_allocator.rs
│  │  │  │  ├─ mod.rs
│  │  │  │  └─ pmm.rs
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
  - Run with the NovaAI model loaded (requires KVM): make run-nova
  - Run with the NovaAI model without KVM (slow): make run-nova-nokvm
  - Build only the sample 16-bit apps into .rlx: make apps16

### Windows
> ⚠️ Native Windows builds are no longer supported as of `v0.03`.
> Please use Linux, macOS, or WSL2 (Recommended) instead.

## 🔗 Links & 🙌 Contributing
- **TikTok:** [tiktok.com/@mainfox.tt](https://www.tiktok.com/@mainfox.tt)
- **Discord:** [discord.gg/Realix](https://discord.gg/D7cATZzAxp)

Contributions of any kind are welcome:

- 🐞 **Report bugs.**
- 💡 **Suggest new features** or improvements.
- 🔧 **Help optimize or refactor code.**

See [Contributing.md](Contributing.md) for guidelines (it documents
the module style, error convention and memory model this codebase follows), or open an issue / reach out via TikTok & Discord.
