// © Realix > Kernel32: Exec Command & Dynamic PATH Executable Runner
// ===================================================================

use crate::drivers::vga::{self, Color};
use crate::rlx_loader;

struct Executable {
    name: &'static str,
    bytes: &'static [u8],
}

// Таблица встроенных исполняемых бинарников в PATH
static EXECUTABLES: &[Executable] = &[
    Executable {
        name: "app32.rlx",
        bytes: &[
            b'R', b'L', b'X', 0x32,
            0x10, 0x00, 0x00, 0x00,
            0x38, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0xB8, 0x01, 0x00, 0x00, 0x00,
            0xBE, 0x23, 0x00, 0x40, 0x00,
            0xCD, 0x80,
            0xB8, 0x03, 0x00, 0x00, 0x00,
            0xCD, 0x80,
            b'[', b'R', b'L', b'X', b'3', b'2', b' ', b'R', b'i', b'n', b'g', b' ', b'3', b' ',
            b'A', b'p', b'p', b']', b' ', b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r', b'o', b'm', b' ',
            b'U', b's', b'e', b'r', b' ', b'M', b'o', b'd', b'e', b' ', b'v', b'i', b'a', b' ',
            b'I', b'N', b'T', b' ', b'0', b'x', b'8', b'0', b'!', b'\r', b'\n', 0x00
        ],
    },
    Executable {
        name: "snake32.rlx",
        bytes: include_bytes!("../../../../build/snake32.rlx"),
    },
    Executable {
        name: "rlxfetch.rlx",
        bytes: include_bytes!("../../../../build/rlxfetch.rlx"),
    },
    Executable {
        name: "exec16.rlx",
        bytes: include_bytes!("../../../../build/exec16.rlx"),
    },
    Executable {
        name: "desktop32.rlx",
        bytes: include_bytes!("../../../../build/desktop32.rlx"),
    },
    Executable {
        name: "calc.rlx",
        bytes: include_bytes!("../../../../build/calc.rlx"),
    },
];

/// Динамический поиск и запуск бинарников из PATH
pub fn find_and_run(cmd: &str) -> bool {
    let name = cmd.trim();
    if name.is_empty() {
        return false;
    }

    for app in EXECUTABLES {
        // Точное совпадение имени файла (например, "snake32.rlx")
        if app.name.eq_ignore_ascii_case(name) {
            vga::print_line("[PATH Resolver] Found executable: ", Color::LightGreen);
            vga::print_line(app.name, Color::LightGreen);
            vga::print_line("\n", Color::LightGreen);
            let _ = rlx_loader::run_rlx32(app.bytes);
            return true;
        }

        // Совпадение по базовому имени без расширения (например, "snake" -> "snake32.rlx")
        let base_name = if let Some(dot_idx) = app.name.find('.') {
            &app.name[..dot_idx]
        } else {
            app.name
        };

        if base_name.eq_ignore_ascii_case(name) {
            vga::print_line("[PATH Resolver] Resolved command via PATH -> ", Color::LightGreen);
            vga::print_line(app.name, Color::LightGreen);
            vga::print_line("\n", Color::LightGreen);
            let _ = rlx_loader::run_rlx32(app.bytes);
            return true;
        }
    }

    false
}

pub fn run(args: &str) {
    let filename = args.trim();
    if filename.is_empty() {
        vga::print_line("Usage: exec <file.rlx> (e.g. exec app32.rlx)\n", Color::Yellow);
        return;
    }

    if !find_and_run(filename) {
        vga::print_line("[Exec Error] Executable file not found in PATH.\n", Color::Red);
    }
}
