// © Realix > Cliff (graphical) - VGA 320x200x256 version of text Cliff
// ================
// ❗️ Только для загрузки через пункт меню "[3] 32-bit Video Mode" - видеорежим
// переключает BIOS ДО перехода в Protected Mode (см. bootloader/initrix/
// switcher.asm: load_kernel32_video), поэтому здесь не нужен рискованный
// переход в Real Mode из Protected Mode во время работы (в отличие от
// удалённого ранее x86::realmode_video) - режим уже установлен, когда
// начинает работать kernel32
// ❗️ Переиспользует ТЕ ЖЕ типы состояния приложений, что и текстовый Cliff
// (commands::cliff::{calc_app, editor, textz, realx, clock, my_pc} - сделаны
// pub(crate) специально для этого) - логика калькулятора/редактора/RealX
// не продублирована, продублирован только рисующий слой (здесь - пикселями
// и растровым шрифтом font.rs, там - vga::write_char_at)
// ❗️ Terminax и Nova сюда не входят - оба печатают через глобальный
// прокручивающийся текстовый вывод (vga::print_line/print_char), которого
// в чистом графическом режиме нет (аппаратный текстовый буфер не
// используется VGA в mode 13h) - это уже отдельная задача (полноценная
// текстовая консоль поверх пикселей), не часть этой правки
// ❗️ Из графического Cliff нет возврата в текстовый shell - тут негде: сам
// видеорежим переключён BIOS до Protected Mode, а обратно переключить его
// без реального перехода в Real Mode (см. выше) нельзя. Escape на столе
// поэтому останавливает систему, а не "выходит" куда-то

mod font;
mod paint;

use crate::commands::cliff::calc_app::{self, CalcApp};
use crate::commands::cliff::clock::{self, ClockApp};
use crate::commands::cliff::editor;
use crate::commands::cliff::my_pc::{self, MyPcApp};
use crate::commands::cliff::realx::lang::RealXOutput;
use crate::commands::cliff::realx::{self, RealXIde};
use crate::commands::cliff::textz::TextZApp;
use crate::drivers::keyboard::{self, Direction, Key};
use crate::drivers::vga::{self, Color};
use paint::PaintApp;

// Размер "клетки" (глиф + межсимвольный интервал) и сетка на экране 320x200
const CELL_W: usize = font::CHAR_ADVANCE;
const CELL_H: usize = font::GLYPH_HEIGHT + 1;
const GRID_COLS: usize = vga::VGA_VIDEO_WIDTH / CELL_W;
const GRID_ROWS: usize = vga::VGA_VIDEO_HEIGHT / CELL_H;

struct IconDef { label: &'static str }
const ICONS: [IconDef; 6] = [
    IconDef { label: "CALC" },
    IconDef { label: "TEXTZ" },
    IconDef { label: "REALX" },
    IconDef { label: "CLOCK" },
    IconDef { label: "MY PC" },
    IconDef { label: "PAINT" },
];
const PAINT_ICON: usize = 5;
const ICON_W: usize = 9;
const ICON_H: usize = 3;
const ICON_GAP: usize = 1;
const ICON_ROW: usize = 2;
const ICON_START_COL: usize = 1;

const CALC_DEFAULT_ROW: usize = 6;
const CALC_DEFAULT_COL: usize = 15;

// Границы размера окна-редактора (TextZ/RealX IDE/Docs/Output) - максимум
// = полный размер буфера (editor::LINE_LEN/MAX_LINES), больше показывать
// нечего; должны укладываться в сетку (GRID_COLS x GRID_ROWS)
const EDITOR_MIN_W: usize = 20;
const EDITOR_MAX_W: usize = editor::LINE_LEN + 4;
const EDITOR_MIN_H: usize = 7;
const EDITOR_MAX_H: usize = editor::MAX_LINES + 3;

// Позиции по умолчанию для окон в полный размер (EDITOR_MAX_W=50) - на
// сетке всего GRID_COLS=53 столбца, так что col не может превышать 3
// (col+width<=GRID_COLS) - небольшое смещение по строкам вместо столбцов
// даёт тот же "слоёный" вид (IDE снизу, Docs/Output чуть выше поверх)
const APP_ROW: usize = 1;
const APP_COL: usize = 1;
const DOCS_ROW: usize = 3;
const DOCS_COL: usize = 2;
const OUTPUT_ROW: usize = 5;
const OUTPUT_COL: usize = 3;

// Paint - фиксированное окно (не двигается/не меняет размер - см.
// handle_top), поэтому нет отдельных MIN/MAX, только один размер.
// Ширина/высота даны с запасом вокруг холста paint::COLS x paint::ROWS
// клеток по paint::CELL_PX пикселей каждая (см. draw_paint_window)
const PAINT_ROW: usize = 1;
const PAINT_COL: usize = 1;
const PAINT_WINDOW_W: usize = 38;
const PAINT_WINDOW_H: usize = 16;

enum Layer {
    Calc(CalcApp),
    TextZ(TextZApp),
    RealXIde(RealXIde),
    RealXDocs,
    RealXOutput(RealXOutput),
    Clock(ClockApp),
    MyPc(MyPcApp),
    Paint(PaintApp),
}

struct Window {
    layer: Layer,
    row: usize,
    col: usize,
    width: usize,
    height: usize,
}

enum Action {
    None,
    Close,
    OpenDocs,
    OpenOutput(RealXOutput),
}

/// Точка входа для загрузки через "[3] 32-bit Video Mode" (см. main.rs) -
/// видеорежим уже включён загрузчиком, здесь только рисуем. Не возвращается
/// (см. заголовок файла) - при выходе останавливает систему
pub fn run() -> ! {
    let mut selected: usize = 0;
    let mut stack: [Option<Window>; 2] = [None, None];
    let mut depth: usize = 0;

    redraw_all(selected, &stack, depth);

    loop {
        if depth == 0 {
            match keyboard::read_key() {
                Key::Escape => break,
                Key::Left => {
                    if selected > 0 { selected -= 1; }
                    redraw_all(selected, &stack, depth);
                }
                Key::Right => {
                    if selected + 1 < ICONS.len() { selected += 1; }
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
                    stack[depth] = Some(Window {
                        layer: Layer::RealXDocs, row: DOCS_ROW, col: DOCS_COL,
                        width: EDITOR_MAX_W, height: EDITOR_MAX_H,
                    });
                    depth += 1;
                }
                Action::OpenOutput(output) => {
                    stack[depth] = Some(Window {
                        layer: Layer::RealXOutput(output), row: OUTPUT_ROW, col: OUTPUT_COL,
                        width: EDITOR_MAX_W, height: EDITOR_MAX_H,
                    });
                    depth += 1;
                }
            }

            redraw_all(selected, &stack, depth);
        }
    }

    halt_with_message();
}

fn halt_with_message() -> ! {
    vga::fill_screen(Color::Black);
    font::draw_text(4, 4, "Left Cliff.", Color::White);
    font::draw_text(4, 4 + CELL_H, "System halted - reboot to restart.", Color::LightGray);
    loop {
        unsafe { core::arch::asm!("cli", "hlt") }
    }
}

fn open_icon(selected: usize) -> Window {
    match selected {
        0 => Window {
            row: CALC_DEFAULT_ROW, col: CALC_DEFAULT_COL,
            width: calc_app::DEFAULT_W, height: calc_app::DEFAULT_H,
            layer: Layer::Calc(CalcApp::new()),
        },
        1 => Window {
            row: APP_ROW, col: APP_COL,
            width: EDITOR_MAX_W, height: EDITOR_MAX_H,
            layer: Layer::TextZ(TextZApp::new()),
        },
        2 => Window {
            row: APP_ROW, col: APP_COL,
            width: EDITOR_MAX_W, height: EDITOR_MAX_H,
            layer: Layer::RealXIde(RealXIde::new()),
        },
        3 => Window {
            row: APP_ROW, col: APP_COL,
            width: EDITOR_MAX_W, height: EDITOR_MAX_H,
            layer: Layer::Clock(ClockApp::new()),
        },
        4 => Window {
            row: APP_ROW, col: APP_COL,
            width: EDITOR_MAX_W, height: EDITOR_MAX_H,
            layer: Layer::MyPc(MyPcApp::new()),
        },
        _ => Window {
            row: PAINT_ROW, col: PAINT_COL,
            width: PAINT_WINDOW_W, height: PAINT_WINDOW_H,
            layer: Layer::Paint(PaintApp::new()),
        },
    }
}

fn resize_bounds(layer: &Layer) -> (usize, usize, usize, usize) {
    match layer {
        Layer::Calc(_) => (calc_app::MIN_W, calc_app::MAX_W, calc_app::MIN_H, calc_app::MAX_H),
        Layer::TextZ(_) | Layer::RealXIde(_) | Layer::RealXDocs | Layer::RealXOutput(_)
        | Layer::Clock(_) | Layer::MyPc(_) =>
            (EDITOR_MIN_W, EDITOR_MAX_W, EDITOR_MIN_H, EDITOR_MAX_H),
        // Paint не двигается/не меняет размер (см. handle_top) - min==max
        // на случай, если сюда всё же дойдёт вызов, ничего не изменится
        Layer::Paint(_) => (PAINT_WINDOW_W, PAINT_WINDOW_W, PAINT_WINDOW_H, PAINT_WINDOW_H),
    }
}

fn apply_move(win: &mut Window, dir: Direction) {
    match dir {
        Direction::Up => win.row = win.row.saturating_sub(1).max(1),
        Direction::Down => win.row = (win.row + 1).min(GRID_ROWS - win.height),
        Direction::Left => win.col = win.col.saturating_sub(1),
        Direction::Right => win.col = (win.col + 1).min(GRID_COLS - win.width),
    }
}

fn apply_resize(win: &mut Window, dir: Direction, min_w: usize, max_w: usize, min_h: usize, max_h: usize) {
    match dir {
        Direction::Up => win.height = win.height.saturating_sub(1).max(min_h),
        Direction::Down => win.height = (win.height + 1).min(max_h),
        Direction::Left => win.width = win.width.saturating_sub(1).max(min_w),
        Direction::Right => win.width = (win.width + 1).min(max_w),
    }

    win.col = win.col.min(GRID_COLS.saturating_sub(win.width));
    win.row = win.row.min(GRID_ROWS.saturating_sub(win.height)).max(1);
}

/// Key::Up/Down/Left/Right -> Direction (для окон, где голые стрелки не
/// заняты ничем другим - см. handle_top)
fn arrow_direction(key: Key) -> Option<Direction> {
    match key {
        Key::Up => Some(Direction::Up),
        Key::Down => Some(Direction::Down),
        Key::Left => Some(Direction::Left),
        Key::Right => Some(Direction::Right),
        _ => None,
    }
}

fn handle_top(win: &mut Window, key: Key) -> Action {
    // Paint - фиксированное окно (по просьбе - не двигается/не меняет
    // размер), поэтому Ctrl+WASD для него намеренно пропускается целиком
    let movable = !matches!(win.layer, Layer::Paint(_));

    match key {
        Key::WindowMove(dir) if movable => { apply_move(win, dir); return Action::None; }
        Key::WindowResize(dir) if movable => {
            let (min_w, max_w, min_h, max_h) = resize_bounds(&win.layer);
            apply_resize(win, dir, min_w, max_w, min_h, max_h);
            return Action::None;
        }
        _ => {}
    }

    // Ctrl+WASD двигает ЛЮБОЕ окно и остаётся единственным способом для
    // TextZ/RealX IDE (голые стрелки там - курсор редактирования). Но
    // Calc/Docs/Output/Clock/My PC стрелками ничего не делают - для них
    // голые стрелки тоже двигают окно
    if matches!(
        win.layer,
        Layer::Calc(_) | Layer::RealXDocs | Layer::RealXOutput(_) | Layer::Clock(_) | Layer::MyPc(_)
    ) {
        if let Some(dir) = arrow_direction(key) {
            apply_move(win, dir);
            return Action::None;
        }
    }

    match &mut win.layer {
        Layer::Calc(calc) => match key {
            Key::Escape => return Action::Close,
            Key::Char(b'\x08') => calc.backspace(),
            Key::Char(b'\n') | Key::Char(b'=') => calc.evaluate(),
            Key::Char(byte) => calc.push(byte),
            _ => {}
        },
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
            Key::Char(b'`') => return Action::OpenOutput(realx::lang::run(&ide.editor, gfx_read_input)),
            Key::Char(byte) if (0x20..=0x7E).contains(&byte) => ide.editor.type_char(byte),
            _ => {}
        },
        Layer::RealXDocs => {
            if let Key::Escape = key { return Action::Close; }
        }
        Layer::RealXOutput(_) => {
            if let Key::Escape = key { return Action::Close; }
        }
        Layer::Clock(_) | Layer::MyPc(_) => {
            if let Key::Escape = key { return Action::Close; }
        }
        Layer::Paint(paint) => match key {
            Key::Escape => return Action::Close,
            Key::Up => paint.move_up(),
            Key::Down => paint.move_down(),
            Key::Left => paint.move_left(),
            Key::Right => paint.move_right(),
            Key::Char(b'\n') => paint.place_square(),
            Key::Char(b' ') => paint.cycle_color(),
            _ => {}
        },
    }
    Action::None
}

/// Чтение строки для input() внутри RealX (см. cliff::realx_read_input -
/// тот же приём, только рисование пикселями): окно вывода в позиции,
/// где его откроет Action::OpenOutput, поле ввода - на следующей клетке
/// после последней выведенной строки
fn gfx_read_input(out: &mut RealXOutput) -> ([u8; realx::lang::INPUT_CAP], usize) {
    draw_output_window(OUTPUT_ROW, OUTPUT_COL, EDITOR_MAX_W, EDITOR_MAX_H, Color::LightGreen, out);

    let content_rows = EDITOR_MAX_H - 3;
    let prompt_row = out.count().saturating_sub(1).min(content_rows - 1);
    let prompt_len = if out.count() > 0 { out.line_str(out.count() - 1).len() } else { 0 };
    let text_row = OUTPUT_ROW + 2 + prompt_row;
    let text_col = OUTPUT_COL + 2 + prompt_len;
    let max_len = (EDITOR_MAX_W - 4).saturating_sub(prompt_len).min(realx::lang::INPUT_CAP);

    let mut buf = [0u8; realx::lang::INPUT_CAP];
    let mut len = 0usize;

    loop {
        // "Курсор" - подчёркивание под текущей позицией (нет аппаратного
        // текстового курсора в графическом режиме)
        draw_text_at(text_row, text_col, "_", Color::LightGreen);
        for i in 0..len {
            draw_char_at(text_row, text_col + i, buf[i] as char, Color::White);
        }

        match keyboard::read_key() {
            Key::Char(b'\n') => break,
            Key::Char(b'\x08') => {
                if len > 0 {
                    len -= 1;
                    draw_char_at(text_row, text_col + len, ' ', Color::White);
                }
            }
            Key::Char(byte) if (0x20..=0x7E).contains(&byte) && len < max_len => {
                buf[len] = byte;
                len += 1;
            }
            _ => {}
        }
    }

    (buf, len)
}

fn redraw_all(selected: usize, stack: &[Option<Window>; 2], depth: usize) {
    draw_desktop(selected);
    for slot in stack.iter().take(depth) {
        if let Some(win) = slot {
            draw_window(win);
        }
    }
}

fn draw_desktop(selected: usize) {
    draw_wallpaper();
    draw_text_at(0, 0, "LR:select Enter:open Esc:halt", Color::White);

    for (i, icon) in ICONS.iter().enumerate() {
        let col = ICON_START_COL + i * (ICON_W + ICON_GAP);
        // Заливаем нутро иконки чёрным перед рамкой - иначе подпись читалась
        // бы поверх обоев (небо/закат/земля), а не однородного фона
        clear_interior(ICON_ROW, col, ICON_W, ICON_H);
        let border = if i == selected { Color::Yellow } else { Color::LightGray };
        draw_box(ICON_ROW, col, ICON_W, ICON_H, border);

        let label_col = col + (ICON_W - icon.label.len()) / 2;
        draw_text_at(ICON_ROW + 1, label_col, icon.label, Color::White);
    }
}

/// Обои рабочего стола - стилизованный закат (небо/солнце/горизонт/земля)
/// несколькими полосами и "звёздами", раскинутыми по детерминированной
/// формуле (в kernel32 нет источника случайности - см. заголовок файла)
/// Обои - тёмно-синий/фиолетовый дизерингованный градиент с диагональным
/// бликом и "глянцевыми" шарами у нижнего правого края (навеяно абстрактными
/// градиентными обоями). VGA mode 13h здесь ограничен 16 именованными
/// цветами (Color) - настоящий плавный градиент недоступен без перепрошивки
/// DAC-палитры, поэтому "дополнительные" оттенки между соседними цветами
/// градиента симулируются дизерингом (чередованием пикселей по
/// псевдослучайному, но детерминированному порогу - без RNG в kernel32)
fn draw_wallpaper() {
    let colors = [Color::Black, Color::Blue, Color::Magenta, Color::LightBlue];
    let bands = colors.len() - 1;

    for y in 0..vga::VGA_VIDEO_HEIGHT {
        // Положение по вертикали в [0, bands) как fixed-point (шаг 1/256)
        let pos = y * bands * 256 / vga::VGA_VIDEO_HEIGHT;
        let band = (pos / 256).min(bands - 1);
        let frac = pos % 256; // насколько близко к следующему цвету полосы

        for x in 0..vga::VGA_VIDEO_WIDTH {
            let dither = (x * 41 + y * 23) % 256;
            let color = if dither < frac { colors[band + 1] } else { colors[band] };
            vga::set_pixel(x, y, color);
        }
    }

    // Широкий диагональный блик (парабола) через весь экран
    let cx: isize = 360;
    let cy: isize = -60;
    for x in 0..vga::VGA_VIDEO_WIDTH {
        let dx = x as isize - cx;
        let y = cy + (dx * dx) / 280;
        if (0..vga::VGA_VIDEO_HEIGHT as isize).contains(&y) {
            vga::set_pixel(x, y as usize, Color::LightCyan);
        }
    }

    // "Глянцевые" шары у нижнего правого края
    draw_disc(305, 250, 95, Color::Blue);
    draw_disc(260, 215, 55, Color::LightBlue);
}

/// Закрашенный круг (простая проверка расстояния - радиус мал, брутфорс ок)
fn draw_disc(cx: usize, cy: usize, r: usize, color: Color) {
    let r2 = (r * r) as isize;
    for dy in -(r as isize)..=(r as isize) {
        for dx in -(r as isize)..=(r as isize) {
            if dx * dx + dy * dy <= r2 {
                let x = cx as isize + dx;
                let y = cy as isize + dy;
                if x >= 0 && y >= 0 {
                    vga::set_pixel(x as usize, y as usize, color);
                }
            }
        }
    }
}

fn draw_window(win: &Window) {
    let (row, col, w, h) = (win.row, win.col, win.width, win.height);
    match &win.layer {
        Layer::Calc(calc) => draw_calc_window(row, col, w, h, calc),
        Layer::TextZ(app) => draw_editor_window(row, col, w, h, "TEXTZ", &app.editor),
        Layer::RealXIde(ide) => draw_editor_window(row, col, w, h, "REALX IDE", &ide.editor),
        Layer::RealXDocs => draw_static_window(row, col, w, h, "REALX DOCS", Color::Yellow, realx::DOCS_TEXT),
        Layer::RealXOutput(out) => {
            let border = if out.error { Color::LightRed } else { Color::LightGreen };
            draw_output_window(row, col, w, h, border, out);
        }
        Layer::Clock(clock) => {
            let (bufs, lens, count) = clock.lines();
            let mut refs: [&str; clock::LINE_COUNT] = [""; clock::LINE_COUNT];
            for i in 0..count {
                refs[i] = core::str::from_utf8(&bufs[i][..lens[i]]).unwrap_or("");
            }
            draw_static_window(row, col, w, h, "CLOCK", Color::Cyan, &refs[..count]);
        }
        Layer::MyPc(my_pc) => {
            let (bufs, lens, count) = my_pc.lines();
            let mut refs: [&str; my_pc::LINE_COUNT] = [""; my_pc::LINE_COUNT];
            for i in 0..count {
                refs[i] = core::str::from_utf8(&bufs[i][..lens[i]]).unwrap_or("");
            }
            draw_static_window(row, col, w, h, "MY PC", Color::Cyan, &refs[..count]);
        }
        Layer::Paint(paint) => draw_paint_window(row, col, paint),
    }
}

/// Paint - холст paint::COLS x paint::ROWS квадратов paint::CELL_PX пикселей
/// каждый, отрисованных заново из состояния (paint.cell()) каждый кадр, плюс
/// обводка-курсор поверх. Позиция/размер окна фиксированы - см. handle_top
fn draw_paint_window(row: usize, col: usize, paint: &PaintApp) {
    let w = PAINT_WINDOW_W;
    let h = PAINT_WINDOW_H;
    clear_interior(row, col, w, h);
    draw_box(row, col, w, h, Color::White);

    draw_text_at(row + 1, col + 1, "PAINT", Color::LightCyan);
    draw_text_at(row + 1, col + w - 10, "Sp:color Ent:draw", Color::DarkGray);

    // Текущий цвет - подпись + закрашенный образец
    draw_text_at(row + 2, col + 1, "COLOR:", Color::LightGray);
    let swatch_x = (col + 8) * CELL_W;
    let swatch_y = (row + 2) * CELL_H;
    vga::draw_rect(swatch_x, swatch_x + 7, swatch_y, swatch_y + CELL_H - 2, paint.current_color());
    draw_text_at(row + 2, col + 10, paint.current_color_name(), Color::White);

    // Холст
    let canvas_x0 = (col + 1) * CELL_W;
    let canvas_y0 = (row + 3) * CELL_H;

    for cy in 0..paint::ROWS {
        for cx in 0..paint::COLS {
            let x0 = canvas_x0 + cx * paint::CELL_PX;
            let y0 = canvas_y0 + cy * paint::CELL_PX;
            let color = paint.cell(cy, cx).unwrap_or(Color::Black);
            vga::draw_rect(x0, x0 + paint::CELL_PX - 2, y0, y0 + paint::CELL_PX - 2, color);
        }
    }

    // Курсор - жёлтая обводка вокруг клетки под ним
    let cx0 = canvas_x0 + paint.cursor_col * paint::CELL_PX;
    let cy0 = canvas_y0 + paint.cursor_row * paint::CELL_PX;
    let cx1 = cx0 + paint::CELL_PX - 2;
    let cy1 = cy0 + paint::CELL_PX - 2;
    vga::draw_hline(cx0, cx1, cy0, Color::Yellow);
    vga::draw_hline(cx0, cx1, cy1, Color::Yellow);
    vga::draw_vline(cx0, cy0, cy1, Color::Yellow);
    vga::draw_vline(cx1, cy0, cy1, Color::Yellow);
}

fn clip(text: &str, max_len: usize) -> &str {
    &text[..text.len().min(max_len)]
}

fn scroll_offset(cursor: usize, visible: usize, total: usize) -> usize {
    let need = cursor.saturating_sub(visible.saturating_sub(1));
    need.min(total.saturating_sub(visible))
}

fn draw_calc_window(row: usize, col: usize, w: usize, h: usize, calc: &CalcApp) {
    clear_interior(row, col, w, h);
    draw_box(row, col, w, h, Color::White);

    let content_w = w - 4;
    draw_text_at(row + 1, col + 2, "CALC", Color::White);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    draw_text_at(row + 2, col + 2, clip(calc.input_str(), content_w), Color::White);

    if calc.error() {
        draw_text_at(row + 3, col + 2, "ERR", Color::LightRed);
    } else if let Some(result) = calc.result() {
        let mut buf: [u8; 12] = [0; 12];
        let text = calc_app::format_result(result, &mut buf);
        draw_text_at(row + 3, col + 2, clip(text, content_w), Color::LightBlue);
    }
}

fn draw_editor_window(row: usize, col: usize, w: usize, h: usize, title: &str, ed: &editor::Editor) {
    clear_interior(row, col, w, h);
    draw_box(row, col, w, h, Color::White);

    draw_text_at(row + 1, col + 2, title, Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);
    if title == "REALX IDE" && w >= 20 {
        draw_text_at(row + 1, col + w - 12, "\\doc `run", Color::LightGray);
    }

    let content_rows = h - 3;
    let content_cols = w - 4;
    let scroll_row = scroll_offset(ed.cur_row, content_rows, editor::MAX_LINES);
    let scroll_col = scroll_offset(ed.cur_col, content_cols, editor::LINE_LEN);

    for r in 0..content_rows {
        let line = ed.line_str(scroll_row + r);
        let start = scroll_col.min(line.len());
        let visible = &line[start..line.len().min(start + content_cols)];
        draw_text_at(row + 2 + r, col + 2, visible, Color::White);
    }

    // "Курсор" - подчёркивание под текущей позицией (нет аппаратного
    // текстового курсора в графическом режиме)
    let cursor_row = row + 2 + (ed.cur_row - scroll_row);
    let cursor_col = col + 2 + (ed.cur_col - scroll_col);
    draw_text_at(cursor_row, cursor_col, "_", Color::Yellow);
}

fn draw_static_window(row: usize, col: usize, w: usize, h: usize, title: &str, border: Color, lines: &[&str]) {
    clear_interior(row, col, w, h);
    draw_box(row, col, w, h, border);

    draw_text_at(row + 1, col + 2, title, Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    let content_rows = h - 3;
    let content_cols = w - 4;
    for (i, line) in lines.iter().enumerate().take(content_rows) {
        draw_text_at(row + 2 + i, col + 2, clip(line, content_cols), Color::White);
    }
}

fn draw_output_window(row: usize, col: usize, w: usize, h: usize, border: Color, out: &RealXOutput) {
    clear_interior(row, col, w, h);
    draw_box(row, col, w, h, border);

    draw_text_at(row + 1, col + 2, "REALX OUTPUT", Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    let content_rows = h - 3;
    let content_cols = w - 4;
    for i in 0..out.count().min(content_rows) {
        draw_text_at(row + 2 + i, col + 2, clip(out.line_str(i), content_cols), Color::White);
    }
}

/// Очистка внутренней области окна пикселями - чёрным (как неявный фон
/// текстовых ячеек в текстовом Cliff), иначе более длинный предыдущий кадр
/// оставлял бы "хвосты" по краям (см. cliff::editor - буфер перезаписи).
/// Именно чёрным, а не белым - иначе светлый текст заголовка/содержимого
/// (White/LightCyan и т.п.) стал бы невидимым на светлом фоне
fn clear_interior(row: usize, col: usize, w: usize, h: usize) {
    if w < 3 || h < 3 { return; }
    let x1 = col * CELL_W + 1;
    let x2 = col * CELL_W + w * CELL_W - 2;
    let y1 = row * CELL_H + 1;
    let y2 = row * CELL_H + h * CELL_H - 2;
    vga::draw_rect(x1, x2, y1, y2, Color::Black);
}

/// Рамка окна/иконки в пикселях (контур, без заливки)
fn draw_box(row: usize, col: usize, w: usize, h: usize, color: Color) {
    let x1 = col * CELL_W;
    let x2 = col * CELL_W + w * CELL_W - 1;
    let y1 = row * CELL_H;
    let y2 = row * CELL_H + h * CELL_H - 1;
    vga::draw_hline(x1, x2, y1, color);
    vga::draw_hline(x1, x2, y2, color);
    vga::draw_vline(x1, y1, y2, color);
    vga::draw_vline(x2, y1, y2, color);
}

fn draw_text_at(row: usize, col: usize, text: &str, color: Color) {
    font::draw_text(col * CELL_W, row * CELL_H, text, color);
}

fn draw_char_at(row: usize, col: usize, c: char, color: Color) {
    font::draw_char(col * CELL_W, row * CELL_H, c, color);
}
