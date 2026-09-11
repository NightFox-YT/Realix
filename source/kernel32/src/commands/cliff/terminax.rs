// © Realix > Cliff: приложение "Terminax" (весь Realix shell из Cliff)
// ================
// ❗️ В отличие от Calc/TextZ/RealX/Clock/My PC, это НЕ окно с рамкой в
// стеке окон Cliff - вывод команд (help, meminfo, matrix, calc, ...) идёт
// через тот же глобальный vga::print_line/print_char, что и обычный
// shell, а не в свой прямоугольник, поэтому Terminax не двигается и не
// меняет размер (Ctrl+WASD), как остальные приложения - cliff::run
// обрабатывает его как отдельный почти-полноэкранный режим (см.
// vga::set_scroll_top - строка 0 зарезервирована под заголовок и не
// участвует в прокрутке содержимого, поэтому она не "уезжает" как обычная
// печатаемая строка, в отличие от предыдущей версии без этого резерва)
// ❗️ Переиспользует shell::execute (та же таблица команд, что и обычный
// Realix >> ) - собственный, более простой построчный ввод (эхо +
// backspace + Enter, без истории/F7) вместо shell::read_line, т.к. та
// завязана на приватную структуру History самого shell.rs

use crate::drivers::keyboard::{self, Key};
use crate::drivers::vga::{self, Color};

const INPUT_CAP: usize = 64;
const TITLE: &str = "TERMINAX - Realix shell  (type 'exit' or Esc to return to Cliff)";

/// Запускает Terminax; возвращается в Cliff по 'exit' или Escape
pub fn run() {
    vga::clear_screen();
    vga::set_scroll_top(1);
    draw_title_bar();

    loop {
        vga::print_line("Realix >> ", Color::Green);
        let (buf, len, escaped) = read_line();

        if escaped {
            vga::print_new_line();
            break;
        }

        let line = core::str::from_utf8(&buf[..len]).unwrap_or("").trim();
        if line == "exit" {
            break;
        }
        if !line.is_empty() {
            crate::shell::execute(line);
        }
        vga::print_new_line_if_needed();
    }

    vga::set_scroll_top(0);
}

/// Заголовок в зарезервированной строке 0 - рисуется один раз, т.к.
/// set_scroll_top(1) уже защищает эту строку от печати/прокрутки ниже
fn draw_title_bar() {
    for col in 0..vga::VGA_TEXT_WIDTH {
        vga::write_char_at(0, col, b' ', Color::Black);
    }
    for (col, &byte) in TITLE.as_bytes().iter().enumerate().take(vga::VGA_TEXT_WIDTH) {
        vga::write_char_at(0, col, byte, Color::LightCyan);
    }
}

/// Чтение строки (эхо, Backspace, Enter) - возвращает (буфер, длину, true
/// если вышли по Escape вместо Enter)
fn read_line() -> ([u8; INPUT_CAP], usize, bool) {
    let mut buf = [0u8; INPUT_CAP];
    let mut pos = 0usize;

    loop {
        match keyboard::read_key() {
            Key::Char(b'\n') => {
                vga::print_new_line();
                return (buf, pos, false);
            }
            Key::Escape => return (buf, pos, true),
            Key::Char(b'\x08') => {
                if pos > 0 {
                    pos -= 1;
                    vga::print_backspace();
                }
            }
            Key::Char(byte) if pos < INPUT_CAP && (0x20..=0x7E).contains(&byte) => {
                buf[pos] = byte;
                pos += 1;
                vga::print_char(byte, Color::LightGray);
            }
            _ => {}
        }
    }
}
