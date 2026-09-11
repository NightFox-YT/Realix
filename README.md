# ℹ️ Realix `v0.11`
![Status](https://img.shields.io/badge/status-latest-brightgreen)
![License](https://img.shields.io/github/license/NightFox-YT/Realix)
![Architecture](https://img.shields.io/badge/architecture-x86-blue)

✅ This version is officially supported and frequently updated by the author.

## 📌 About
Realix is a **hybrid OS** for the x86 architecture. The bootloader and 16-bit kernel are written in x86 assembly (NASM), the 32-bit kernel is written in Rust (`no_std`).
It supports a built-in boot switcher with four modes: a **stable** 16-bit Real Mode kernel, a **stable** 32-bit Protected Mode kernel, and 🆕 **Nightly** variants of each, where new, less-tested features land first (disk access, RLX apps, MS-DOS `.COM` support, etc.) before they're promoted to stable.

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
- Interactive CPU mode selector (`switcher.asm`, now 4 boot options)
- Kernel16 (stable, `[1]` in the boot menu, `source/kernel16/`):
   - Command-line interface with history (Up and down arrows)
   - 16-bit syscall interface (`int 0x80`) with an RLX executable loader — user-mode 16-bit apps live under apps/ and run via `exec`
   - Commands (grouped by category):
     - base: `help`, `clear/cls`, `echo [t]`, `about`, `beep`, `meminfo`, `uptime`, `sysinfo`
     - numbers: `calc` (positive integers only), `hex`, `fib`
     - text: `len`, `upper`, `lower`, `reverse`, `repeat`, `ascii`
     - system: `regs`, `time`, `date`, `vga`, `panic`
     - power: `reboot`, `shutdown`
     - fat12: `ls`, `load`, `type` (Print a file as text), `hexdump`, `exec` (Run a 16-bit rlx app)
   - Panic and exception handlers with a diagnostic screen
   - Network interface card (NIC) driver
- 🆕 Kernel16 Nightly (`[4]` in the boot menu, `source/kernel16-nightly/`) - everything from stable, plus:
   - A from-scratch DOS API compatibility layer (`int 20h`/`int 21h`, own implementation - no Microsoft/FreeDOS code involved) supporting the classic `.COM` memory model (code+data+stack in one 64 KB segment, entry at offset 0x100): program termination (`00h`/`4Ch`), console I/O (`01h`/`02h`/`08h`/`09h`/`0Ah`/`0Bh`), DOS version query (`30h`). Unknown functions return `CF=1, AX=1` like real DOS instead of crashing.
   - Minimal PSP (Program Segment Prefix) builder with command-line passthrough (`msexec <file> [args]` forwards args DOS-style)
   - `msexec <file.com>` - loads and runs a real MS-DOS `.COM` application; termination jumps back to the shell via a saved stack pointer (Robust even if the program leaves its own stack in a mess)
   - ❌ Not yet implemented: `.EXE` support, file I/O (`3Ch`/`3Dh`/`3Fh`/...), memory management (`48h`/`49h`/`4Ah`), FCBs
   - Sample program: `dosapps/hello.asm` → `hello.com` (Greets you by name, exercises string output, buffered input, char echo, and clean exit)
- Kernel32 (stable, `[2]` in the boot menu, `source/kernel32/`):
   - VGA text mode (80×25)
   - Command-line interface
     - `help`, `echo [t]`, `clear/cls`
     - `uptime` - Show uptime in seconds
     - `meminfo` - Show memory information
     - `matrix` - Fun animation
     - `reboot`, `shutdown`
   - Advanced GDT with TSS
   - Advanced IDT with ISR (Exceptions, IRQ0 - PIT, IRQ1 - Keyboard)
   - PIC remap, spurious interrupt (IRQ7/IRQ15) detection
   - Frame allocator built from the E820 memory map passed by the bootloader
   - NovaAI experiment: run a local Qwen 2.5 0.5B GGUF model from inside kernel32 via the run-nova / run-nova-nokvm Make targets
   - No disk/filesystem access (see Kernel32 Nightly)
- 🆕 Kernel32 Nightly (`[3]` in the boot menu, `source/kernel32-nightly/`) - everything from stable, plus:
   - Shell with history (Up/Down arrows) and Shift support (uppercase, shifted symbols)
     - base: `about`, `beep`, `sysinfo`
     - numbers: `calc` (16-bit unsigned, +/-/*/ with overflow & div-by-zero checks), `hex`, `fib <0-46>`
     - text: `len`, `upper`, `lower`, `reverse`, `repeat`, `ascii`
     - system: `regs` (CPU registers snapshot), `time`/`date` (CMOS RTC), `vga` (colored rectangles demo, text-mode), `panic` (manual panic screen)
     - fat12: `ls`, `load`, `type`, `hexdump`, `exec` (Run a 32-bit .rlx app)
   - PC speaker driver (PIT channel 2 + port 0x61) and CMOS RTC driver (direct port I/O, no BIOS)
   - Native floppy disk controller driver (i8272A/82077AA via ports 0x3F0-0x3F7 + ISA DMA channel 2, IRQ6) - no BIOS int 0x13 in protected mode, so this talks to the FDC hardware directly
   - Read-only FAT12 filesystem (BPB parsed live from the boot sector, root dir search, 12-bit cluster chain walking) on top of the floppy driver
   - 32-bit syscall interface (`int 0x80`, same call numbers as kernel16, plus `SYS_GET_UPTIME`/`SYS_GET_MEMINFO`/`SYS_GET_VERSION`) with an RLX executable loader — 32-bit user-mode apps live under apps/ (`hello32.asm`, `rfetch.asm`) and run via `exec`, flat-memory `call`/`ret` convention (no segment tricks needed, unlike 16-bit; apps must preserve EBX/ESI/EDI/EBP per the C ABI). Runs in Ring 0 for now — GDT already has unused Ring 3 descriptors + TSS for a future real privilege-isolated version
   - IDT: added IRQ6 (Floppy)
   - Rust panic handler with a diagnostic screen (reason + file:line), matching kernel16's panic screen style
   - `realixfetch` (`apps/rfetch.asm`) - a neofetch-style system info tool: ASCII logo + OS/kernel/uptime/memory (via syscalls) + real CPU vendor (via `cpuid`)

### ⏳ Upcoming Features (v0.12)
- Promote proven Nightly features (kernel16 & kernel32) to stable once they've had more real-world testing
- Kernel16 Nightly: DOS file I/O (`3Dh`/`3Fh`/`42h`/...), memory management (`48h`/`49h`/`4Ah`), maybe basic `.EXE` support
- Kernel32: NIC driver port (rtl8139)
- Kernel32: VMM module
- Kernel32: Ring 3 privilege isolation for RLX apps (GDT/TSS groundwork already in place)
- Add new docs for kernel16 & kernel32.

### ❌ Current Limitations
- No dynamic memory allocator
- No write operations FAT12 support (All kernels are read-only on disk)
- No advanced networking stack
- Disk access is CHS-only (no `int 13h` extensions in kernel16; kernel32 Nightly's floppy driver is CHS-native)
- Kernel32 Nightly's RLX loader runs apps in Ring 0 (no privilege isolation yet, same trust model as kernel16)
- Kernel32 (stable) has no disk/filesystem access at all - use Nightly for `ls`/`load`/`type`/`hexdump`/`exec`
- Kernel16 Nightly's `msexec` only supports simple console `.COM` programs (No file I/O, no memory management, no `.EXE`) - see DOS API subset above

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
│  ├─ hello16.asm
│  ├─ hello32.asm
│  └─ rfetch.asm            # realixfetch - neofetch-style system info tool
├─ dosapps/                 # 🆕 Real MS-DOS .COM sources (kernel16-nightly msexec)
│  └─ hello.asm             # -> hello.com
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
│  ├─ kernel16/                  # Stable (boot menu [1])
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
│  ├─ kernel16-nightly/          # 🆕 Nightly (boot menu [4]) - same layout as kernel16, plus:
│  │  ├─ commands/file-system/
│  │  │  └─ msexec.asm           # MS-DOS .COM loader command
│  │  ├─ dos/
│  │  │  ├─ psp.asm              # PSP builder
│  │  │  └─ int21.asm            # int 20h/21h dispatcher (Own implementation)
│  │  ├─ ... (+ everything from kernel16/)
│  │  └─ Makefile
│  ├─ kernel32/                  # Stable (boot menu [2])
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
│  │  │  │  ├─ mod.rs
│  │  │  │  └─ pic.rs
│  │  │  ├─ main.rs
│  │  │  ├─ shell.rs
│  │  │  └─ utils.rs
│  │  ├─ linker.ld               # Shared with kernel32-nightly (same load address)
│  │  └─ Makefile
│  ├─ kernel32-nightly/          # 🆕 Nightly (boot menu [3]) - same layout as kernel32, plus:
│  │  ├─ nova-models/
│  │  ├─ src/
│  │  │  ├─ commands/
│  │  │  │  ├─ about.rs
│  │  │  │  ├─ beep.rs
│  │  │  │  ├─ calc.rs
│  │  │  │  ├─ clock.rs
│  │  │  │  ├─ disk.rs
│  │  │  │  ├─ exec.rs        # 32-bit RLX loader
│  │  │  │  ├─ hexdump.rs
│  │  │  │  ├─ load.rs
│  │  │  │  ├─ ls.rs
│  │  │  │  ├─ numbers.rs
│  │  │  │  ├─ panic.rs
│  │  │  │  ├─ regs.rs
│  │  │  │  ├─ sysinfo.rs
│  │  │  │  ├─ text.rs
│  │  │  │  ├─ type_cmd.rs
│  │  │  │  ├─ vga_demo.rs
│  │  │  │  └─ ... (+ everything from kernel32/commands/)
│  │  │  ├─ drivers/
│  │  │  │  ├─ dma.rs
│  │  │  │  ├─ floppy.rs      # FDC driver (no BIOS)
│  │  │  │  ├─ rtc.rs         # CMOS RTC (no BIOS)
│  │  │  │  ├─ speaker.rs
│  │  │  │  └─ ... (+ everything from kernel32/drivers/)
│  │  │  ├─ fs/
│  │  │  │  ├─ fat12.rs
│  │  │  │  └─ mod.rs
│  │  │  ├─ x86/
│  │  │  │  ├─ syscall.rs     # int 0x80 for 32-bit RLX apps
│  │  │  │  └─ ... (+ everything from kernel32/x86/)
│  │  │  └─ ... (+ memory/, main.rs, shell.rs, utils.rs)
│  │  └─ Makefile              # Builds --bin kernel32-nightly from the root Cargo.toml
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
  - Build only the sample apps into .rlx (16 and 32-bit): make apps16

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
