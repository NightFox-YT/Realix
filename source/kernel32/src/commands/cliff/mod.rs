// © Realix > Command: Cliff (tiny text-mode desktop environment)
// (10.09.26) v0.3
// ================
// ❗️ Полностью в штатном текстовом режиме 80x25 (vga::write_char_at) - без
// переключения видеорежима вообще (прежняя пиксельная версия и команда
// gfx убраны, см. историю коммитов - при желании их несложно вернуть)
// ❗️ Мышь не поддерживается - нет PS/2-драйвера мыши. Навигация клавиатурой:
//    на столе - стрелки Влево/Вправо выбирают иконку (видно по цвету рамки),
//    Enter/Space открывают выбранную; Escape на столе выходит из Cliff.
//    В приложении Calc - стрелки двигают окно; в TextZ/RealX IDE - стрелки
//    двигают курсор редактирования (окно на месте - слишком большое, чтобы
//    таскать было практично). Escape в приложении закрывает его окно.
// ❗️ До двух окон одновременно в стеке (RealX: IDE внизу + Docs/Output
//    поверх, "новое окно" по '\'/'`' - см. Layer/Action ниже); остальные
//    приложения используют только один уровень стека

mod calc_app;
mod editor;
mod realx;
mod textz;

use crate::drivers::keyboard::{self, Key};
use crate::drivers::vga::{self, Color};
use calc_app::CalcApp;
use realx::RealXIde;
use realx::lang::RealXOutput;
use textz::TextZApp;

// Иконки рабочего стола
struct IconDef { label: &'static str }
const ICONS: [IconDef; 3] = [
    IconDef { label: "CALC" },
    IconDef { label: "TEXTZ" },
    IconDef { label: "REALX" },
];
const ICON_W: usize = 10;
const ICON_H: usize = 3;
const ICON_GAP: usize = 2;
const ICON_ROW: usize = 2;
const ICON_START_COL: usize = 2;

// Позиции окон по умолчанию (Calc - маленькое и подвижное; TextZ/RealX -
// побольше, стрелки внутри них двигают курсор, а не окно)
const CALC_DEFAULT_ROW: usize = 8;
const CALC_DEFAULT_COL: usize = 30;
const APP_ROW: usize = 1;
const APP_COL: usize = 2;
const DOCS_ROW: usize = 3;
const DOCS_COL: usize = 10;
const OUTPUT_ROW: usize = 5;
const OUTPUT_COL: usize = 16;

// Куда "прятать" аппаратный текстовый курсор, когда его позиция не имеет
// смысла (стол, Calc) - подальше от содержимого, в угол экрана
const PARKED_CURSOR_ROW: usize = vga::VGA_TEXT_HEIGHT - 1;
const PARKED_CURSOR_COL: usize = vga::VGA_TEXT_WIDTH - 1;

enum Layer {
    Calc(CalcApp),
    TextZ(TextZApp),
    RealXIde(RealXIde),
    RealXDocs,
    RealXOutput(RealXOutput),
}

struct Window {
    layer: Layer,
    row: usize,
    col: usize,
}

enum Action {
    None,
    Close,
    OpenDocs,
    OpenOutput(RealXOutput),
}

/// Точка входа команды "cliff"
pub fn run() {
    let mut selected: usize = 0;
    let mut stack: [Option<Window>; 2] = [None, None];
    let mut depth: usize = 0;

    redraw_all(selected, &stack, depth);

    loop {
        if depth == 0 {
            match keyboard::read_key() {
                Key::Escape => break,
                Key::Left => {
                    selected = selected.saturating_sub(1);
                    redraw_all(selected, &stack, depth);
                }
                Key::Right => {
                    selected = (selected + 1).min(ICONS.len() - 1);
                    redraw_all(selected, &stack, depth);
                }
                Key::Char(b'\n') | Key::Char(b' ') => {
                    stack[0] = Some(open_icon(selected));
                    depth = 1;
                    redraw_all(selected, &stack, depth);
                }
                _ => {}
            }
        } else {
            let key = keyboard::read_key();
            let action = handle_top(stack[depth - 1].as_mut().unwrap(), key);

            match action {
                Action::None => {}
                Action::Close => {
                    stack[depth - 1] = None;
                    depth -= 1;
                }
                Action::OpenDocs => {
                    stack[depth] = Some(Window { layer: Layer::RealXDocs, row: DOCS_ROW, col: DOCS_COL });
                    depth += 1;
                }
                Action::OpenOutput(output) => {
                    stack[depth] = Some(Window { layer: Layer::RealXOutput(output), row: OUTPUT_ROW, col: OUTPUT_COL });
                    depth += 1;
                }
            }

            redraw_all(selected, &stack, depth);
        }
    }

    vga::clear_screen();
    vga::print_line("Left Cliff.\n", Color::LightGray);
}

fn open_icon(selected: usize) -> Window {
    match selected {
        0 => Window { row: CALC_DEFAULT_ROW, col: CALC_DEFAULT_COL, layer: Layer::Calc(CalcApp::new()) },
        1 => Window { row: APP_ROW, col: APP_COL, layer: Layer::TextZ(TextZApp::new()) },
        _ => Window { row: APP_ROW, col: APP_COL, layer: Layer::RealXIde(RealXIde::new()) },
    }
}

/// Обрабатывает клавишу для верхнего (активного) окна в стеке
fn handle_top(win: &mut Window, key: Key) -> Action {
    match &mut win.layer {
        Layer::Calc(calc) => {
            match key {
                Key::Escape => return Action::Close,
                Key::Up => win.row = win.row.saturating_sub(1).max(1),
                Key::Down => win.row = (win.row + 1).min(vga::VGA_TEXT_HEIGHT - calc_app::WINDOW_H),
                Key::Left => win.col = win.col.saturating_sub(1),
                Key::Right => win.col = (win.col + 1).min(vga::VGA_TEXT_WIDTH - calc_app::WINDOW_W),
                Key::Char(b'\x08') => calc.backspace(),
                Key::Char(b'\n') | Key::Char(b'=') => calc.evaluate(),
                Key::Char(byte) => calc.push(byte),
                Key::F7 => {}
            }
        }
        Layer::TextZ(app) => match key {
            Key::Escape => return Action::Close,
            Key::Up => app.editor.move_up(),
            Key::Down => app.editor.move_down(),
            Key::Left => app.editor.move_left(),
            Key::Right => app.editor.move_right(),
            Key::Char(b'\x08') => app.editor.backspace(),
            Key::Char(b'\n') => app.editor.newline(),
            Key::Char(byte) if (0x20..=0x7E).contains(&byte) => app.editor.type_char(byte),
            _ => {}
        },
        Layer::RealXIde(ide) => match key {
            Key::Escape => return Action::Close,
            Key::Up => ide.editor.move_up(),
            Key::Down => ide.editor.move_down(),
            Key::Left => ide.editor.move_left(),
            Key::Right => ide.editor.move_right(),
            Key::Char(b'\x08') => ide.editor.backspace(),
            Key::Char(b'\n') => ide.editor.newline(),
            Key::Char(b'\\') => return Action::OpenDocs,
            Key::Char(b'`') => return Action::OpenOutput(realx::lang::run(&ide.editor)),
            Key::Char(byte) if (0x20..=0x7E).contains(&byte) => ide.editor.type_char(byte),
            _ => {}
        },
        Layer::RealXDocs => {
            if let Key::Escape = key { return Action::Close; }
        }
        Layer::RealXOutput(_) => {
            if let Key::Escape = key { return Action::Close; }
        }
    }
    Action::None
}

/// Полная перерисовка кадра: стол, затем все окна стека снизу вверх
fn redraw_all(selected: usize, stack: &[Option<Window>; 2], depth: usize) {
    draw_desktop(selected);
    for slot in stack.iter().take(depth) {
        if let Some(win) = slot {
            draw_window(win);
        }
    }
}

fn draw_desktop(selected: usize) {
    vga::clear_screen();
    vga::set_cursor_pos(PARKED_CURSOR_ROW, PARKED_CURSOR_COL);

    draw_text_at(0, 0, "Cliff - Left/Right: select  Enter/Space: open  Esc: close/exit", Color::LightGray);

    for (i, icon) in ICONS.iter().enumerate() {
        let col = ICON_START_COL + i * (ICON_W + ICON_GAP);
        let border = if i == selected { Color::Yellow } else { Color::DarkGray };
        draw_box_text(ICON_ROW, col, ICON_W, ICON_H, border);

        let label_col = col + (ICON_W - icon.label.len()) / 2;
        draw_text_at(ICON_ROW + 1, label_col, icon.label, Color::White);
    }
}

fn draw_window(win: &Window) {
    match &win.layer {
        Layer::Calc(calc) => draw_calc_window(win.row, win.col, calc),
        Layer::TextZ(app) => draw_editor_window(win.row, win.col, "TEXTZ", &app.editor),
        Layer::RealXIde(ide) => draw_editor_window(win.row, win.col, "REALX IDE", &ide.editor),
        Layer::RealXDocs => draw_docs_window(win.row, win.col),
        Layer::RealXOutput(out) => draw_output_window(win.row, win.col, out),
    }
}

fn draw_calc_window(row: usize, col: usize, calc: &CalcApp) {
    let w = calc_app::WINDOW_W;
    let h = calc_app::WINDOW_H;
    clear_interior(row, col, w, h);
    draw_box_text(row, col, w, h, Color::White);

    draw_text_at(row + 1, col + 2, "CALC", Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    draw_text_at(row + 2, col + 2, calc.input_str(), Color::White);

    if calc.error() {
        draw_text_at(row + 3, col + 2, "ERR", Color::LightRed);
    } else if let Some(result) = calc.result() {
        let mut buf: [u8; 12] = [0; 12];
        let text = calc_app::format_result(result, &mut buf);
        draw_text_at(row + 3, col + 2, text, Color::LightBlue);
    }

    vga::set_cursor_pos(PARKED_CURSOR_ROW, PARKED_CURSOR_COL);
}

/// Общее окно редактора (TextZ и RealX IDE делят один и тот же вид)
fn draw_editor_window(row: usize, col: usize, title: &str, ed: &editor::Editor) {
    let w = editor::LINE_LEN + 4;
    let h = editor::MAX_LINES + 3;
    clear_interior(row, col, w, h);
    draw_box_text(row, col, w, h, Color::White);

    draw_text_at(row + 1, col + 2, title, Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);
    if title == "REALX IDE" {
        draw_text_at(row + 1, col + w - 12, "\\doc `run", Color::DarkGray);
    }

    for r in 0..editor::MAX_LINES {
        draw_text_at(row + 2 + r, col + 2, ed.line_str(r), Color::White);
    }

    vga::set_cursor_pos(row + 2 + ed.cur_row, col + 2 + ed.cur_col);
}

fn draw_docs_window(row: usize, col: usize) {
    let w = editor::LINE_LEN + 4;
    let h = editor::MAX_LINES + 3;
    clear_interior(row, col, w, h);
    draw_box_text(row, col, w, h, Color::Yellow);

    draw_text_at(row + 1, col + 2, "REALX DOCS", Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    for (i, line) in realx::DOCS_TEXT.iter().enumerate().take(editor::MAX_LINES) {
        draw_text_at(row + 2 + i, col + 2, line, Color::White);
    }

    vga::set_cursor_pos(PARKED_CURSOR_ROW, PARKED_CURSOR_COL);
}

fn draw_output_window(row: usize, col: usize, out: &RealXOutput) {
    let w = editor::LINE_LEN + 4;
    let h = editor::MAX_LINES + 3;
    clear_interior(row, col, w, h);
    let border = if out.error { Color::LightRed } else { Color::LightGreen };
    draw_box_text(row, col, w, h, border);

    draw_text_at(row + 1, col + 2, "REALX OUTPUT", Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    for i in 0..out.count().min(editor::MAX_LINES) {
        draw_text_at(row + 2 + i, col + 2, out.line_str(i), Color::White);
    }

    vga::set_cursor_pos(PARKED_CURSOR_ROW, PARKED_CURSOR_COL);
}

/// Очистка внутренней области окна - иначе более длинный предыдущий кадр
/// оставлял бы "хвосты" по краям (см. cliff::editor - буфер перезаписи)
fn clear_interior(row: usize, col: usize, w: usize, h: usize) {
    for r in 1..h - 1 {
        for c in 1..w - 1 {
            vga::write_char_at(row + r, col + c, b' ', Color::Black);
        }
    }
}

/// Отрисовка прямоугольной рамки из символов рамки (только контур)
fn draw_box_text(row: usize, col: usize, w: usize, h: usize, color: Color) {
    vga::write_char_at(row, col, b'+', color);
    vga::write_char_at(row, col + w - 1, b'+', color);
    vga::write_char_at(row + h - 1, col, b'+', color);
    vga::write_char_at(row + h - 1, col + w - 1, b'+', color);

    for c in 1..w - 1 {
        vga::write_char_at(row, col + c, b'-', color);
        vga::write_char_at(row + h - 1, col + c, b'-', color);
    }
    for r in 1..h - 1 {
        vga::write_char_at(row + r, col, b'|', color);
        vga::write_char_at(row + r, col + w - 1, b'|', color);
    }
}

/// Вывод строки символов в заданной позиции (row/col в знакоместах)
fn draw_text_at(row: usize, col: usize, text: &str, color: Color) {
    for (i, &byte) in text.as_bytes().iter().enumerate() {
        vga::write_char_at(row, col + i, byte, color);
    }
}
