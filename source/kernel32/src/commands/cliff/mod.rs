// © Realix > Command: Cliff (tiny text-mode desktop environment)
// (10.09.26) v0.3
// ================
// ❗️ Полностью в штатном текстовом режиме 80x25 (vga::write_char_at) - без
// переключения видеорежима вообще (прежняя пиксельная версия и команда
// gfx убраны, см. историю коммитов - при желании их несложно вернуть)
// ❗️ Мышь не поддерживается - нет PS/2-драйвера мыши. Навигация клавиатурой:
//    на столе - стрелки Влево/Вправо выбирают иконку (видно по цвету рамки),
//    Enter/Space открывают выбранную; Escape на столе выходит из Cliff.
//    В приложении - стрелки - это курсор/ввод (Calc: цифры и операторы;
//    TextZ/RealX IDE: курсор редактирования); Escape закрывает окно.
//    Управление ЛЮБЫМ окном (любое приложение, единообразно) - Ctrl+WASD:
//    Ctrl+W/A/S/D двигает окно, Ctrl+Shift+W/A/S/D меняет его размер (см.
//    keyboard::Key::WindowMove/WindowResize, apply_move/apply_resize,
//    resize_bounds). Именно Ctrl (не голый WASD) - иначе нельзя было бы
//    печатать буквы w/a/s/d в TextZ/RealX IDE. TextZ/RealX IDE ограничены
//    снизу видом (клетка редактора не может уменьшиться до нуля) и сверху
//    полным размером буфера (больше показывать всё равно нечего - см.
//    editor.rs). Уменьшенное окно редактора показывает viewport вокруг
//    курсора (scroll_offset), а не весь буфер - Docs/Output (без курсора)
//    всегда показывают буфер с начала
// ❗️ До двух окон одновременно в стеке (RealX: IDE внизу + Docs/Output
//    поверх, "новое окно" по '\'/'`' - см. Layer/Action ниже); остальные
//    приложения используют только один уровень стека

// pub(crate) - переиспользуются commands::cliff_gfx (те же типы состояния
// приложений, другой рисующий слой - см. её заголовок)
pub(crate) mod calc_app;
pub(crate) mod clock;
pub(crate) mod editor;
pub(crate) mod my_pc;
pub(crate) mod realx;
mod terminax;
pub(crate) mod textz;

use crate::drivers::keyboard::{self, Direction, Key};
use crate::drivers::rtc;
use crate::drivers::vga::{self, Color};
use calc_app::CalcApp;
use clock::ClockApp;
use my_pc::MyPcApp;
use realx::RealXIde;
use realx::lang::RealXOutput;
use textz::TextZApp;

// Иконки рабочего стола - последние две (Terminax/Nova) не являются
// оконными приложениями (см. Layer/open_icon ниже - для них Enter/Space
// в run() не создаёт Window, а напрямую вызывает полноэкранный режим)
// glyph - маленький "пиктограммный" символ внутри рамки иконки, отдельно от
// label - имени приложения, напечатанного отдельной строкой ПОД рамкой
struct IconDef { glyph: &'static str, label: &'static str }
const ICONS: [IconDef; 7] = [
    IconDef { glyph: "[=]", label: "CALC" },
    IconDef { glyph: "[T]", label: "TEXTZ" },
    IconDef { glyph: "{X}", label: "REALX" },
    IconDef { glyph: "(O)", label: "CLOCK" },
    IconDef { glyph: "[#]", label: "MY PC" },
    IconDef { glyph: ">_", label: "TERMNX" },
    IconDef { glyph: "(*)", label: "NOVA" },
];
const TERMINAX_ICON: usize = 5;
const NOVA_ICON: usize = 6;

const ICON_W: usize = 9;
const ICON_H: usize = 3;
const ICON_GAP: usize = 1;
const ICON_ROW: usize = 2;
const ICON_START_COL: usize = 2;
const ICONS_PER_ROW: usize = 6;

// Периодическая перерисовка даже без нажатий (см. keyboard::read_key_timeout)
// - иначе часы на панели "застревали" бы на минуте последнего нажатия.
// 2000 тиков PIT (100Гц) = ~20 сек - запас на случай, если таймаут сработал
// сразу после смены минуты (иначе воспринимаемая задержка могла бы быть
// почти вдвое дольше самого периода проверки)
const IDLE_REDRAW_TICKS: u32 = 2000;

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

// Границы размера для TextZ/RealX IDE/Docs/Output (все делят один и тот же
// вид окна-редактора) - максимум = "естественный" полный размер буфера
// (editor::LINE_LEN/MAX_LINES) - больше показывать всё равно нечего
const EDITOR_MIN_W: usize = 24;
const EDITOR_MAX_W: usize = editor::LINE_LEN + 4;
const EDITOR_MIN_H: usize = 7;
const EDITOR_MAX_H: usize = editor::MAX_LINES + 3;

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
    Clock(ClockApp),
    MyPc(MyPcApp),
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

/// Точка входа команды "cliff"
pub fn run() {
    let mut selected: usize = 0;
    let mut stack: [Option<Window>; 2] = [None, None];
    let mut depth: usize = 0;

    redraw_all(selected, &stack, depth);

    loop {
        if depth == 0 {
            // Таймаут (не только реальная клавиша) - чтобы часы на панели
            // не "застревали" на минуте, в которую последний раз нажимали
            // клавишу (см. IDLE_REDRAW_TICKS)
            let key = match keyboard::read_key_timeout(IDLE_REDRAW_TICKS) {
                Some(key) => key,
                None => { redraw_all(selected, &stack, depth); continue; }
            };

            match key {
                Key::Escape => break,
                Key::Left => {
                    if selected % ICONS_PER_ROW > 0 { selected -= 1; }
                    redraw_all(selected, &stack, depth);
                }
                Key::Right => {
                    if selected % ICONS_PER_ROW < ICONS_PER_ROW - 1 && selected + 1 < ICONS.len() {
                        selected += 1;
                    }
                    redraw_all(selected, &stack, depth);
                }
                Key::Up => {
                    if selected >= ICONS_PER_ROW { selected -= ICONS_PER_ROW; }
                    redraw_all(selected, &stack, depth);
                }
                Key::Down => {
                    if selected + ICONS_PER_ROW < ICONS.len() { selected += ICONS_PER_ROW; }
                    redraw_all(selected, &stack, depth);
                }
                Key::Char(b'\n') | Key::Char(b' ') => {
                    // Terminax/Nova - полноэкранные режимы, не окна Cliff
                    // (см. заголовок файла) - вызываются напрямую, без Window
                    if selected == TERMINAX_ICON {
                        terminax::run();
                    } else if selected == NOVA_ICON {
                        unsafe { crate::commands::nova_ai::BC("-a"); }
                    } else {
                        stack[0] = Some(open_icon(selected));
                        depth = 1;
                    }
                    redraw_all(selected, &stack, depth);
                }
                _ => {}
            }
        } else {
            let key = match keyboard::read_key_timeout(IDLE_REDRAW_TICKS) {
                Some(key) => key,
                None => { redraw_all(selected, &stack, depth); continue; }
            };
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

    vga::clear_screen();
    vga::print_line("Left Cliff.\n", Color::LightGray);
}

/// Только для иконок 0-4 - Terminax/Nova (5/6) не создают Window (см. run())
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
        _ => Window {
            row: APP_ROW, col: APP_COL,
            width: EDITOR_MAX_W, height: EDITOR_MAX_H,
            layer: Layer::MyPc(MyPcApp::new()),
        },
    }
}

/// Границы изменения размера для типа окна (см. apply_resize)
fn resize_bounds(layer: &Layer) -> (usize, usize, usize, usize) {
    match layer {
        Layer::Calc(_) => (calc_app::MIN_W, calc_app::MAX_W, calc_app::MIN_H, calc_app::MAX_H),
        Layer::TextZ(_) | Layer::RealXIde(_) | Layer::RealXDocs | Layer::RealXOutput(_)
        | Layer::Clock(_) | Layer::MyPc(_) =>
            (EDITOR_MIN_W, EDITOR_MAX_W, EDITOR_MIN_H, EDITOR_MAX_H),
    }
}

/// Ctrl+WASD - двигает окно (в пределах экрана, не залезая на строку
/// подсказки в верхней строке)
fn apply_move(win: &mut Window, dir: Direction) {
    match dir {
        Direction::Up => win.row = win.row.saturating_sub(1).max(1),
        Direction::Down => win.row = (win.row + 1).min(vga::VGA_TEXT_HEIGHT - win.height),
        Direction::Left => win.col = win.col.saturating_sub(1),
        Direction::Right => win.col = (win.col + 1).min(vga::VGA_TEXT_WIDTH - win.width),
    }
}

/// Ctrl+Shift+WASD - меняет размер окна (в пределах min/max для его типа);
/// после изменения окно подвинуто, чтобы не вылезти за экран
fn apply_resize(win: &mut Window, dir: Direction, min_w: usize, max_w: usize, min_h: usize, max_h: usize) {
    match dir {
        Direction::Up => win.height = win.height.saturating_sub(1).max(min_h),
        Direction::Down => win.height = (win.height + 1).min(max_h),
        Direction::Left => win.width = win.width.saturating_sub(1).max(min_w),
        Direction::Right => win.width = (win.width + 1).min(max_w),
    }

    win.col = win.col.min(vga::VGA_TEXT_WIDTH.saturating_sub(win.width));
    win.row = win.row.min(vga::VGA_TEXT_HEIGHT.saturating_sub(win.height)).max(1);
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

/// Обрабатывает клавишу для верхнего (активного) окна в стеке
fn handle_top(win: &mut Window, key: Key) -> Action {
    match key {
        Key::WindowMove(dir) => { apply_move(win, dir); return Action::None; }
        Key::WindowResize(dir) => {
            let (min_w, max_w, min_h, max_h) = resize_bounds(&win.layer);
            apply_resize(win, dir, min_w, max_w, min_h, max_h);
            return Action::None;
        }
        _ => {}
    }

    // Ctrl+WASD двигает ЛЮБОЕ окно (см. выше) и остаётся единственным
    // способом для TextZ/RealX IDE, чьи голые стрелки - курсор
    // редактирования. Но Calc/Docs/Output/Clock/My PC стрелками вообще
    // ничего не делают - им незачем отбирать стрелки под что-то другое,
    // так что для них голые стрелки тоже двигают окно (более привычно)
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
        Layer::Calc(calc) => {
            match key {
                Key::Escape => return Action::Close,
                Key::Char(b'\x08') => calc.backspace(),
                Key::Char(b'\n') | Key::Char(b'=') => calc.evaluate(),
                Key::Char(byte) => calc.push(byte),
                _ => {}
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
            Key::Char(b'`') => return Action::OpenOutput(realx::lang::run(&ide.editor, realx_read_input)),
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
    }
    Action::None
}

/// Читает строку с клавиатуры для input() внутри выполняемой программы
/// RealX (см. realx::lang::EvalCtx) - рисует окно вывода РОВНО в той позиции
/// и размере, где его откроет Action::OpenOutput (OUTPUT_ROW/OUTPUT_COL/
/// EDITOR_MAX_W/EDITOR_MAX_H), затем показывает вводимый текст на следующей
/// клетке после последней выведенной строки (подсказки от input(), если
/// была, уже в `out` к этому моменту - см. lang::parse_input_call) и
/// блокирующе читает клавиатуру до Enter. Безопасно вызывать вложенно из
/// run(), т.к. read_key() - обычный синхронный опрос, как и везде в kernel32
fn realx_read_input(out: &mut RealXOutput) -> ([u8; realx::lang::INPUT_CAP], usize) {
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
        vga::set_cursor_pos(text_row, text_col + len);
        match keyboard::read_key() {
            Key::Char(b'\n') => break,
            Key::Char(b'\x08') => {
                if len > 0 {
                    len -= 1;
                    vga::write_char_at(text_row, text_col + len, b' ', Color::White);
                }
            }
            Key::Char(byte) if (0x20..=0x7E).contains(&byte) && len < max_len => {
                buf[len] = byte;
                vga::write_char_at(text_row, text_col + len, byte, Color::White);
                len += 1;
            }
            _ => {}
        }
    }

    (buf, len)
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

    draw_text_at(0, 0, "Select:LR Open:Enter/Space Close:Esc Move:Ctrl+WASD Resize:Ctrl+Shift+WASD", Color::LightGray);

    for (i, icon) in ICONS.iter().enumerate() {
        let col = ICON_START_COL + (i % ICONS_PER_ROW) * (ICON_W + ICON_GAP);
        // +2 (не +1): под рамкой теперь ещё и отдельная строка подписи -
        // см. ICON_H ниже строки рамки
        let row = ICON_ROW + (i / ICONS_PER_ROW) * (ICON_H + 2);
        let selected_here = i == selected;
        let border = if selected_here { Color::Yellow } else { Color::DarkGray };
        draw_box_text(row, col, ICON_W, ICON_H, border);

        // Выбранная иконка - глиф тоже жёлтый (текстовый режим не даёт
        // залить фон, см. заголовок файла - только цвет символов)
        let glyph_color = if selected_here { Color::Yellow } else { Color::White };
        let glyph_col = col + (ICON_W - icon.glyph.len()) / 2;
        draw_text_at(row + 1, glyph_col, icon.glyph, glyph_color);

        // Подпись - отдельной строкой ПОД рамкой, не внутри неё
        let label_col = col + ICON_W.saturating_sub(icon.label.len()) / 2;
        draw_text_at(row + ICON_H, label_col, icon.label, Color::LightGray);
    }

    draw_taskbar();
}

/// Панель внизу экрана - имя системы слева, часы (RTC) справа. Обновляется
/// при каждой перерисовке стола (т.е. на любую клавишу - см. заголовок
/// clock.rs про то же ограничение: нет таймерного пробуждения главного цикла)
fn draw_taskbar() {
    let row = vga::VGA_TEXT_HEIGHT - 1;
    for col in 0..vga::VGA_TEXT_WIDTH {
        vga::write_char_at(row, col, b' ', Color::DarkGray);
    }
    draw_text_at(row, 1, "CLIFF", Color::Yellow);

    let now = rtc::now();
    let mut buf = [0u8; 5];
    let clock = format_hms(&now, &mut buf);
    draw_text_at(row, vga::VGA_TEXT_WIDTH - clock.len() - 1, clock, Color::White);
}

/// "HH:MM:SS" в буфер фиксированного размера
fn format_hms<'a>(now: &rtc::DateTime, buf: &'a mut [u8; 5]) -> &'a str {
    buf[0] = b'0' + now.hour / 10;
    buf[1] = b'0' + now.hour % 10;
    buf[2] = b':';
    buf[3] = b'0' + now.minute / 10;
    buf[4] = b'0' + now.minute % 10;
    core::str::from_utf8(buf).unwrap_or("--:--")
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
    }
}

/// Обрезка строки (только ASCII в этом проекте - байтовый срез безопасен)
fn clip(text: &str, max_len: usize) -> &str {
    &text[..text.len().min(max_len)]
}

/// Смещение "прокрутки" - минимальное, чтобы курсор оставался виден в окне
/// шириной/высотой `visible` из `total` строк/столбцов буфера
fn scroll_offset(cursor: usize, visible: usize, total: usize) -> usize {
    let need = cursor.saturating_sub(visible.saturating_sub(1));
    need.min(total.saturating_sub(visible))
}

fn draw_calc_window(row: usize, col: usize, w: usize, h: usize, calc: &CalcApp) {
    clear_interior(row, col, w, h);
    draw_box_text(row, col, w, h, Color::White);

    let content_w = w - 4;
    draw_text_at(row + 1, col + 2, "CALC", Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    draw_text_at(row + 2, col + 2, clip(calc.input_str(), content_w), Color::White);

    if calc.error() {
        draw_text_at(row + 3, col + 2, "ERR", Color::LightRed);
    } else if let Some(result) = calc.result() {
        let mut buf: [u8; 12] = [0; 12];
        let text = calc_app::format_result(result, &mut buf);
        draw_text_at(row + 3, col + 2, clip(text, content_w), Color::LightBlue);
    }

    vga::set_cursor_pos(PARKED_CURSOR_ROW, PARKED_CURSOR_COL);
}

/// Общее окно редактора (TextZ и RealX IDE делят один и тот же вид) -
/// показывает viewport вокруг курсора, если окно меньше полного буфера
fn draw_editor_window(row: usize, col: usize, w: usize, h: usize, title: &str, ed: &editor::Editor) {
    clear_interior(row, col, w, h);
    draw_box_text(row, col, w, h, Color::White);

    draw_text_at(row + 1, col + 2, title, Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);
    if title == "REALX IDE" && w >= 24 {
        draw_text_at(row + 1, col + w - 12, "\\doc `run", Color::DarkGray);
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

    vga::set_cursor_pos(row + 2 + (ed.cur_row - scroll_row), col + 2 + (ed.cur_col - scroll_col));
}

/// Окно со статичным текстом (RealX Docs) - без курсора, всегда с начала
fn draw_static_window(row: usize, col: usize, w: usize, h: usize, title: &str, border: Color, lines: &[&str]) {
    clear_interior(row, col, w, h);
    draw_box_text(row, col, w, h, border);

    draw_text_at(row + 1, col + 2, title, Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    let content_rows = h - 3;
    let content_cols = w - 4;
    for (i, line) in lines.iter().enumerate().take(content_rows) {
        draw_text_at(row + 2 + i, col + 2, clip(line, content_cols), Color::White);
    }

    vga::set_cursor_pos(PARKED_CURSOR_ROW, PARKED_CURSOR_COL);
}

fn draw_output_window(row: usize, col: usize, w: usize, h: usize, border: Color, out: &RealXOutput) {
    clear_interior(row, col, w, h);
    draw_box_text(row, col, w, h, border);

    draw_text_at(row + 1, col + 2, "REALX OUTPUT", Color::LightCyan);
    draw_text_at(row + 1, col + w - 4, "[X]", Color::LightRed);

    let content_rows = h - 3;
    let content_cols = w - 4;
    for i in 0..out.count().min(content_rows) {
        draw_text_at(row + 2 + i, col + 2, clip(out.line_str(i), content_cols), Color::White);
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
