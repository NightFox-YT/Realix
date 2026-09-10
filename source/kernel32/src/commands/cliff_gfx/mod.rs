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
use crate::drivers::mouse;
use crate::drivers::vga::{self, Color};
use paint::PaintApp;

// Размер "клетки" (глиф + межсимвольный интервал) - абсолютный, в пикселях
const CELL_W: usize = font::CHAR_ADVANCE;
const CELL_H: usize = font::GLYPH_HEIGHT + 1;

/// Сетка в знакоместах - функции, а не const, просто для симметрии с
/// vga::video_width/height (единственный источник размера холста)
fn grid_cols() -> usize { vga::video_width() / CELL_W }
fn grid_rows() -> usize { vga::video_height() / CELL_H }

// Иконки рисуются пикселями по номеру (см. draw_icon), а не текстовым
// глифом - "kind" это просто индекс в draw_icon's match, не связанный с
// позицией в этом массиве (на случай, если порядок когда-то разъедется)
struct IconDef { kind: usize, label: &'static str }
const ICONS: [IconDef; 6] = [
    IconDef { kind: 0, label: "CALC" },
    IconDef { kind: 1, label: "TEXTZ" },
    IconDef { kind: 2, label: "REALX" },
    IconDef { kind: 3, label: "CLOCK" },
    IconDef { kind: 4, label: "MY PC" },
    IconDef { kind: 5, label: "PAINT" },
];
const PAINT_ICON: usize = 5;
const ICON_W: usize = 9;
// 5 (не 3): нужна настоящая графическая область под пиктограмму, а не одна
// текстовая строка - см. draw_icon (рисует внутри ICON_H-2 "внутренних" строк)
const ICON_H: usize = 5;
const ICON_GAP: usize = 1;
const ICON_ROW: usize = 2;
const ICON_START_COL: usize = 1;
// grid_cols() = 53 - 5 иконок ровно влезают в ряд ((ICON_W+ICON_GAP)*5=50);
// 6-я (Paint) переносится на второй ряд, как и в текстовом Cliff
const ICONS_PER_ROW: usize = 5;

// Периодическая перерисовка даже без нажатий - см. cliff::IDLE_REDRAW_TICKS
// (тот же механизм read_key_timeout). Здесь НАМНОГО короче (~30мс вместо
// ~20сек) - главная причина уже не часы на панели, а курсор мыши: он должен
// обновляться быстро, а read_key_timeout - единственное место, где цикл
// вообще просыпается и перечитывает drivers::mouse::poll() (см. run())
const IDLE_REDRAW_TICKS: u32 = 3;

const CALC_DEFAULT_ROW: usize = 6;
const CALC_DEFAULT_COL: usize = 15;

// Границы размера окна-редактора (TextZ/RealX IDE/Docs/Output) - максимум
// = полный размер буфера (editor::LINE_LEN/MAX_LINES), больше показывать
// нечего; должны укладываться в сетку (grid_cols() x grid_rows())
const EDITOR_MIN_W: usize = 20;
const EDITOR_MAX_W: usize = editor::LINE_LEN + 4;
const EDITOR_MIN_H: usize = 7;
const EDITOR_MAX_H: usize = editor::MAX_LINES + 3;

// Позиции по умолчанию для окон в полный размер (EDITOR_MAX_W=50) -
// grid_cols() всего 53 столбца, так что col не может превышать 3
// (col+width<=grid_cols()) - небольшое смещение по строкам вместо столбцов
// даёт тот же "слоёный" вид (IDE снизу, Docs/Output чуть выше поверх)
const APP_ROW: usize = 1;
const APP_COL: usize = 1;
const DOCS_ROW: usize = 3;
const DOCS_COL: usize = 2;
const OUTPUT_ROW: usize = 5;
const OUTPUT_COL: usize = 3;

// Paint - позиция фиксирована (не двигается - см. handle_top про
// movable/resizable по отдельности), но МЕНЯЕТСЯ в размере: увеличение
// открывает больше клеток одного и того же буфера paint::MAX_COLS x
// paint::MAX_ROWS (см. paint_visible_size), а не создаёт холст заново
const PAINT_ROW: usize = 1;
const PAINT_COL: usize = 1;
const PAINT_DEFAULT_W: usize = 38;
const PAINT_DEFAULT_H: usize = 16;
const PAINT_MIN_W: usize = 20;
const PAINT_MIN_H: usize = 8;
// Верхняя граница - с запасом под paint::MAX_COLS/MAX_ROWS (см. её
// заголовок и paint_visible_size - формула перевода размера окна в
// видимые клетки); дальше не пускает всё равно ёмкость самого буфера
const PAINT_MAX_W: usize = 70;
const PAINT_MAX_H: usize = 36;

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

// "Задача" - одно открытое с рабочего стола приложение, со своим собственным
// маленьким стеком окон (windows/depth - тот же принцип, что был раньше
// единственным на весь Cliff: второй уровень нужен только RealX IDE для
// Docs/Output поверх себя, см. handle_top). Несколько задач могут быть
// открыты ОДНОВРЕМЕННО (см. MAX_TASKS) - это и есть "многозадачность" Cliff:
// кооперативная, без вытеснения (никакого отдельного потока/треда на задачу
// нет - каждая просто хранит своё состояние между кадрами, пока не в фокусе,
// см. run()/redraw_all), но приложения не теряют состояние, пока свёрнуты
struct Task {
    windows: [Option<Window>; 2],
    depth: usize,
    // Индекс в ICONS - какое приложение это, для подписи кнопки на панели
    // задач (см. draw_taskbar) и чтобы повторный клик по той же иконке на
    // рабочем столе переключался на уже открытую задачу, а не плодил вторую
    icon: usize,
}

// 4 одновременно открытых задачи - с запасом под все 6 приложений (Paint,
// самое частое "долгоживущее", + пара более коротких сессий одновременно),
// но не все 6 сразу - см. open_or_focus_task про вытеснение, если слотов не
// хватило
const MAX_TASKS: usize = 4;

/// Точка входа для загрузки через "[3] 32-bit Video Mode" (см. main.rs) -
/// видеорежим уже включён загрузчиком, здесь только рисуем. Не возвращается
/// (см. заголовок файла) - при выходе останавливает систему
pub fn run() -> ! {
    let mut selected: usize = 0;
    let mut tasks: [Option<Task>; MAX_TASKS] = [None, None, None, None];
    // None - показывается рабочий стол (иконки кликабельны/выбираемы
    // стрелками); Some(i) - задача tasks[i] в фокусе (получает
    // клавиатуру/клики, рисуется поверх остальных - см. redraw_all).
    // Другие открытые задачи продолжают существовать (их состояние не
    // теряется) и видны на панели задач, просто не интерактивны, пока не
    // выбраны кликом по своей кнопке там же (см. taskbar_task_hit)
    let mut focused: Option<usize> = None;

    let mut mouse_x: i32 = (vga::video_width() / 2) as i32;
    let mut mouse_y: i32 = (vga::video_height() / 2) as i32;
    let mut mouse_was_down = false;

    // Смещение (в пикселях) между точкой клика и левым верхним углом окна,
    // пока идёт перетаскивание за заголовок - см. цикл ниже. None - не тащим
    let mut dragging: Option<(i32, i32)> = None;

    redraw_all(selected, &tasks, focused, mouse_x, mouse_y);

    loop {
        // Таймаут короткий (в отличие от текстового Cliff) - без него
        // движение мыши было бы почти невидимо: IRQ12 будит hlt в
        // read_key_timeout, но сам цикл там смотрит только очередь
        // клавиатуры, так что без частого таймаута курсор обновлялся бы
        // только раз в IDLE_REDRAW_TICKS (было ~20 сек, для мыши это
        // неприемлемо) или когда ЗАОДНО пришла клавиша
        let key = keyboard::read_key_timeout(IDLE_REDRAW_TICKS);

        // Опрос мыши - независимо от того, что разбудило цикл (клавиша,
        // движение мыши или таймаут). Y инвертирован: у PS/2 положительный
        // dy - движение ВВЕРХ, а экранные координаты растут вниз
        let (dx, dy, left_down) = mouse::poll();
        mouse_x = (mouse_x + dx).clamp(0, vga::video_width() as i32 - 1);
        mouse_y = (mouse_y - dy).clamp(0, vga::video_height() as i32 - 1);
        let mouse_clicked = left_down && !mouse_was_down;
        mouse_was_down = left_down;

        // Панель задач кликабельна ВСЕГДА - и с рабочего стола, и изнутри
        // задачи (переключиться на другую задачу, не закрывая текущую).
        // Клик, который попал сюда, дальше НЕ обрабатывается как клик по
        // иконке/окну (см. переопределение mouse_clicked ниже) - панель
        // задач "перехватывает" его первой
        let mut taskbar_consumed = false;
        if mouse_clicked {
            if home_button_hit(mouse_x, mouse_y) {
                focused = None;
                taskbar_consumed = true;
            } else if let Some(slot) = taskbar_task_hit(&tasks, mouse_x, mouse_y) {
                focused = Some(slot);
                taskbar_consumed = true;
            }
        }
        let mouse_clicked = mouse_clicked && !taskbar_consumed;

        match focused {
            None => {
                // Актуально только пока была задача в фокусе - если фокус
                // сброшен (напр. кликом по "CLIFF"), пока шло
                // перетаскивание, не даём смещению "просочиться" на
                // СЛЕДУЮЩУЮ сфокусированную задачу
                dragging = None;

                if mouse_clicked {
                    if let Some(icon) = icon_at_point(mouse_x, mouse_y) {
                        // Клик по уже выбранной иконке - открыть (как
                        // Enter/Space); по другой - просто выбрать её (как
                        // двойной клик на обычном столе, но без учёта
                        // времени между кликами - тут нет часов для этого)
                        if icon == selected {
                            focused = Some(open_or_focus_task(&mut tasks, selected));
                        } else {
                            selected = icon;
                        }
                    }
                }

                match key {
                    Some(Key::Escape) => break,
                    Some(Key::Left) => {
                        if selected % ICONS_PER_ROW > 0 { selected -= 1; }
                    }
                    Some(Key::Right) => {
                        if selected % ICONS_PER_ROW < ICONS_PER_ROW - 1 && selected + 1 < ICONS.len() {
                            selected += 1;
                        }
                    }
                    Some(Key::Up) => {
                        if selected >= ICONS_PER_ROW { selected -= ICONS_PER_ROW; }
                    }
                    Some(Key::Down) => {
                        if selected + ICONS_PER_ROW < ICONS.len() { selected += ICONS_PER_ROW; }
                    }
                    Some(Key::Char(b'\n')) | Some(Key::Char(b' ')) => {
                        focused = Some(open_or_focus_task(&mut tasks, selected));
                    }
                    _ => {}
                }
            }
            Some(task_idx) => {
                let task = tasks[task_idx].as_mut().unwrap();
                let win = task.windows[task.depth - 1].as_mut().unwrap();

                // Paint не рисует "[X]" (закрывается только по Escape - см.
                // draw_paint_window) и не двигается (по просьбе - позиция
                // фиксирована, но РАЗМЕР - нет, см. handle_top); без этой
                // проверки клик в той же строке (где у Paint просто текстовая
                // подсказка) закрывал бы его без всякой видимой кнопки, а
                // перетаскивание двигало бы окно, которое должно быть
                // неподвижным
                let movable = !matches!(win.layer, Layer::Paint(_));

                // Рисование в Paint мышью - зажатая ЛКМ над холстом ставит
                // квадрат под курсором; таскать - рисовать непрерывно.
                // Клетка вычисляется ДО заимствования win.layer как mut ниже -
                // paint_cell_at_point берёт весь Window (для col/row/width),
                // а не только layer
                let hovered_paint_cell = if left_down { paint_cell_at_point(win, mouse_x, mouse_y) } else { None };
                if let (Layer::Paint(paint), Some((row, col))) = (&mut win.layer, hovered_paint_cell) {
                    paint.set_cursor(row, col);
                    paint.place_square();
                }

                // Перетаскивание за заголовок (не по "[X]") - начинается кликом
                // по строке заголовка, продолжается, пока зажата ЛКМ, и
                // заканчивается её отпусканием. Координаты мыши (пиксели)
                // пересчитываются в "клетки" (CELL_W/CELL_H) окна, те же
                // единицы, что и Ctrl+WASD (apply_move)
                if movable {
                    if mouse_clicked && title_bar_hit(win, mouse_x, mouse_y) && !close_button_hit(win, mouse_x, mouse_y) {
                        let offset_x = mouse_x - (win.col * CELL_W) as i32;
                        let offset_y = mouse_y - (win.row * CELL_H) as i32;
                        dragging = Some((offset_x, offset_y));
                    }

                    if let Some((offset_x, offset_y)) = dragging {
                        if left_down {
                            let new_col = ((mouse_x - offset_x).max(0) as usize) / CELL_W;
                            let new_row = ((mouse_y - offset_y).max(0) as usize) / CELL_H;
                            win.col = new_col.min(grid_cols().saturating_sub(win.width));
                            win.row = new_row.min(grid_rows().saturating_sub(win.height)).max(1);
                        } else {
                            dragging = None;
                        }
                    }
                }

                let action = if mouse_clicked && movable && close_button_hit(win, mouse_x, mouse_y) {
                    Action::Close
                } else if let Some(key) = key {
                    handle_top(win, key, (mouse_x, mouse_y, left_down))
                } else {
                    Action::None
                };

                match action {
                    Action::None => {}
                    Action::Close => {
                        task.windows[task.depth - 1] = None;
                        task.depth -= 1;
                        if task.depth == 0 {
                            tasks[task_idx] = None;
                            focused = None;
                        }
                    }
                    Action::OpenDocs => {
                        task.windows[task.depth] = Some(Window {
                            layer: Layer::RealXDocs, row: DOCS_ROW, col: DOCS_COL,
                            width: EDITOR_MAX_W, height: EDITOR_MAX_H,
                        });
                        task.depth += 1;
                    }
                    Action::OpenOutput(output) => {
                        task.windows[task.depth] = Some(Window {
                            layer: Layer::RealXOutput(output), row: OUTPUT_ROW, col: OUTPUT_COL,
                            width: EDITOR_MAX_W, height: EDITOR_MAX_H,
                        });
                        task.depth += 1;
                    }
                }
            }
        }

        redraw_all(selected, &tasks, focused, mouse_x, mouse_y);
    }

    halt_with_message();
}

/// Уже открытая задача с той же иконкой - просто в фокус (не плодим вторую
/// копию того же приложения); иначе - новая задача в свободный слот. Если
/// свободных слотов не осталось (все MAX_TASKS заняты) - вытесняется
/// последний слот (простая эвикция, без LRU - для 6 приложений и 4 слотов
/// это редкий, не особо болезненный случай)
fn open_or_focus_task(tasks: &mut [Option<Task>; MAX_TASKS], icon: usize) -> usize {
    for (i, slot) in tasks.iter().enumerate() {
        if let Some(task) = slot {
            if task.icon == icon {
                return i;
            }
        }
    }

    for (i, slot) in tasks.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(Task { windows: [Some(open_icon(icon)), None], depth: 1, icon });
            return i;
        }
    }

    let last = MAX_TASKS - 1;
    tasks[last] = Some(Task { windows: [Some(open_icon(icon)), None], depth: 1, icon });
    last
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
            width: PAINT_DEFAULT_W, height: PAINT_DEFAULT_H,
            layer: Layer::Paint(PaintApp::new()),
        },
    }
}

/// Верхняя граница размера окна для типа окна, не считая min/max самого
/// приложения - не даёт окну стать шире/выше текущей сетки экрана (нужно
/// отдельно от apply_resize/её же min/max, т.к. сетка зависит от видеорежима)
fn resize_bounds(layer: &Layer) -> (usize, usize, usize, usize) {
    match layer {
        Layer::Calc(_) => (calc_app::MIN_W, calc_app::MAX_W, calc_app::MIN_H, calc_app::MAX_H),
        Layer::TextZ(_) | Layer::RealXIde(_) | Layer::RealXDocs | Layer::RealXOutput(_)
        | Layer::Clock(_) | Layer::MyPc(_) =>
            (EDITOR_MIN_W, EDITOR_MAX_W, EDITOR_MIN_H, EDITOR_MAX_H),
        // Позиция Paint фиксирована (см. handle_top - `movable`), но размер
        // - нет: рост окна открывает больше клеток того же буфера (см.
        // paint::MAX_COLS/MAX_ROWS, paint_visible_size) - верхняя граница
        // ограничена ещё и текущей сеткой экрана (не даёт вылезти за экран,
        // раз окно нельзя подвинуть, чтобы это исправить)
        Layer::Paint(_) => (
            PAINT_MIN_W, PAINT_MAX_W.min(grid_cols().saturating_sub(PAINT_COL)),
            PAINT_MIN_H, PAINT_MAX_H.min(grid_rows().saturating_sub(PAINT_ROW)),
        ),
    }
}

fn apply_move(win: &mut Window, dir: Direction) {
    match dir {
        Direction::Up => win.row = win.row.saturating_sub(1).max(1),
        Direction::Down => win.row = (win.row + 1).min(grid_rows() - win.height),
        Direction::Left => win.col = win.col.saturating_sub(1),
        Direction::Right => win.col = (win.col + 1).min(grid_cols() - win.width),
    }
}

fn apply_resize(win: &mut Window, dir: Direction, min_w: usize, max_w: usize, min_h: usize, max_h: usize) {
    match dir {
        Direction::Up => win.height = win.height.saturating_sub(1).max(min_h),
        Direction::Down => win.height = (win.height + 1).min(max_h),
        Direction::Left => win.width = win.width.saturating_sub(1).max(min_w),
        Direction::Right => win.width = (win.width + 1).min(max_w),
    }

    win.col = win.col.min(grid_cols().saturating_sub(win.width));
    win.row = win.row.min(grid_rows().saturating_sub(win.height)).max(1);
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

fn handle_top(win: &mut Window, key: Key, mouse: (i32, i32, bool)) -> Action {
    // Paint - позиция фиксирована (по просьбе), но размер - нет: рост окна
    // открывает больше клеток холста (см. paint.rs, paint_visible_size).
    // movable/resizable отдельно друг от друга именно из-за Paint - для
    // всех остальных приложений они всегда совпадают
    let movable = !matches!(win.layer, Layer::Paint(_));
    let resizable = true;

    match key {
        Key::WindowMove(dir) if movable => { apply_move(win, dir); return Action::None; }
        Key::WindowResize(dir) if resizable => {
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

    // Нужно ДО заимствования win.layer как mut ниже - Layer::Paint(paint)
    // не даёт одновременно читать win.width/height через win напрямую
    let (win_w, win_h) = (win.width, win.height);

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
            Key::Char(b'`') => return Action::OpenOutput(realx::lang::run(&ide.editor, mouse, true, gfx_read_input)),
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
            Key::Down => {
                let (_, visible_rows) = paint_visible_size(win_w, win_h);
                paint.move_down(visible_rows);
            }
            Key::Left => paint.move_left(),
            Key::Right => {
                let (visible_cols, _) = paint_visible_size(win_w, win_h);
                paint.move_right(visible_cols);
            }
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

/// Позиция иконки в сетке (grid row, grid col) по её индексу - общая формула
/// для отрисовки (draw_desktop) и попадания курсора (icon_at_point)
fn icon_grid_pos(i: usize) -> (usize, usize) {
    let col = ICON_START_COL + (i % ICONS_PER_ROW) * (ICON_W + ICON_GAP);
    let row = ICON_ROW + (i / ICONS_PER_ROW) * (ICON_H + 2);
    (row, col)
}

fn point_in_rect(px: i32, py: i32, x0: usize, y0: usize, w: usize, h: usize) -> bool {
    px >= x0 as i32 && px < (x0 + w) as i32 && py >= y0 as i32 && py < (y0 + h) as i32
}

/// Иконка под точкой курсора (мышь) - хитбокс включает подпись под рамкой
/// (ICON_H+1 строк), не только саму рамку
fn icon_at_point(px: i32, py: i32) -> Option<usize> {
    for i in 0..ICONS.len() {
        let (row, col) = icon_grid_pos(i);
        if point_in_rect(px, py, col * CELL_W, row * CELL_H, ICON_W * CELL_W, (ICON_H + 1) * CELL_H) {
            return Some(i);
        }
    }
    None
}

/// true, если точка курсора попадает в кнопку "[X]" верхнего окна (см.
/// draw_calc_window/draw_editor_window/... - все рисуют её в одном месте:
/// строка row+1, последние 3 знакоместа с правого края рамки)
fn close_button_hit(win: &Window, px: i32, py: i32) -> bool {
    let x0 = (win.col + win.width - 4) * CELL_W;
    let y0 = (win.row + 1) * CELL_H;
    point_in_rect(px, py, x0, y0, 3 * CELL_W, CELL_H)
}

/// Вся строка заголовка (та же строка row+1, что и "[X]") - используется
/// для начала перетаскивания окна мышью (см. run()); вызывающая сторона
/// сама решает, что делать, если попадание ЕЩЁ И в close_button_hit
fn title_bar_hit(win: &Window, px: i32, py: i32) -> bool {
    point_in_rect(px, py, win.col * CELL_W, (win.row + 1) * CELL_H, win.width * CELL_W, CELL_H)
}

/// Клетка холста Paint под точкой курсора, если она внутри холста - см.
/// draw_paint_window про ту же геометрию (canvas_x0/canvas_y0)
fn paint_cell_at_point(win: &Window, px: i32, py: i32) -> Option<(usize, usize)> {
    let canvas_x0 = ((win.col + 1) * CELL_W) as i32;
    let canvas_y0 = ((win.row + 3) * CELL_H) as i32;

    if px < canvas_x0 || py < canvas_y0 {
        return None;
    }

    let (visible_cols, visible_rows) = paint_visible_size(win.width, win.height);
    let cx = (px - canvas_x0) as usize / paint::CELL_PX;
    let cy = (py - canvas_y0) as usize / paint::CELL_PX;

    if cx < visible_cols && cy < visible_rows { Some((cy, cx)) } else { None }
}

/// Сколько клеток холста Paint видно (и доступно для рисования) при данном
/// размере окна - растёт вместе с окном, ограничено ёмкостью буфера
/// (paint::MAX_COLS/MAX_ROWS). "-2"/"-4" - рамка+отступ по бокам, рамка+
/// заголовок+строка цвета сверху (см. draw_paint_window - тот же отсчёт
/// координат холста, canvas_x0/canvas_y0)
fn paint_visible_size(win_w: usize, win_h: usize) -> (usize, usize) {
    let content_w_px = win_w.saturating_sub(2) * CELL_W;
    let content_h_px = win_h.saturating_sub(4) * CELL_H;
    let cols = (content_w_px / paint::CELL_PX).clamp(1, paint::MAX_COLS);
    let rows = (content_h_px / paint::CELL_PX).clamp(1, paint::MAX_ROWS);
    (cols, rows)
}

const CURSOR_SIZE: usize = 12;
const CURSOR_MID: usize = CURSOR_SIZE / 2;

/// Курсор мыши - сплошная "стрелка"-ромб (расширяется до середины, потом
/// сужается обратно - предыдущая версия только расширялась и обрывалась на
/// середине, выглядело как обрезанный наполовину треугольник) с чёрной
/// обводкой (видна на любом фоне), остриём в точке (x,y). Рисуется поверх
/// всего остального кадра (см. redraw_all) - как и все остальные элементы
/// Cliff, каждый кадр заново, поэтому отдельно стирать предыдущую позицию
/// не нужно
fn draw_cursor(x: i32, y: i32) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize, y as usize);
    for row in 0..CURSOR_SIZE {
        let width = if row <= CURSOR_MID { row + 1 } else { CURSOR_SIZE - row };
        for col in 0..width {
            let on_edge = col == 0 || col == width - 1;
            let color = if on_edge { Color::Black } else { Color::White };
            vga::set_pixel(x + col, y + row, color);
        }
    }
}

/// Рисует ВСЕ открытые задачи (не только сфокусированную) - несфокусированные
/// первыми, сфокусированная последней (поверх остальных, см. draw_window) -
/// это то, что делает открытые-но-не-в-фокусе задачи видимыми одновременно
/// с активной, а не скрытыми, пока не выбраны
fn redraw_all(selected: usize, tasks: &[Option<Task>; MAX_TASKS], focused: Option<usize>, mouse_x: i32, mouse_y: i32) {
    draw_desktop(selected, tasks, focused);

    for (i, slot) in tasks.iter().enumerate() {
        if Some(i) == focused { continue; }
        if let Some(task) = slot {
            draw_task_windows(task);
        }
    }
    if let Some(i) = focused {
        if let Some(task) = &tasks[i] {
            draw_task_windows(task);
        }
    }

    draw_cursor(mouse_x, mouse_y);
}

fn draw_task_windows(task: &Task) {
    for slot in task.windows.iter().take(task.depth) {
        if let Some(win) = slot {
            draw_window(win);
        }
    }
}

fn draw_desktop(selected: usize, tasks: &[Option<Task>; MAX_TASKS], focused: Option<usize>) {
    draw_wallpaper();
    draw_text_at(0, 0, "Arrows:select Enter:open Esc:halt", Color::White);

    for (i, icon) in ICONS.iter().enumerate() {
        let (row, col) = icon_grid_pos(i);
        let selected_here = i == selected;

        // Выбранная иконка - залитый фон (в отличие от текстового Cliff,
        // тут пиксели, так что настоящая заливка возможна), а не только
        // цвет рамки
        let fill = if selected_here { Color::Blue } else { Color::Black };
        fill_interior(row, col, ICON_W, ICON_H, fill);
        let border = if selected_here { Color::Yellow } else { Color::LightGray };
        draw_box(row, col, ICON_W, ICON_H, border);

        // Пиктограмма - в "внутренних" строках рамки (между верхней и
        // нижней границей, т.е. ICON_H-2 строк по высоте)
        let content_x0 = col * CELL_W + 4;
        let content_y0 = row * CELL_H + CELL_H;
        let content_w = ICON_W * CELL_W - 8;
        let content_h = (ICON_H - 2) * CELL_H;
        draw_icon(icon.kind, content_x0, content_y0, content_w, content_h);

        // Подпись - отдельной строкой ПОД рамкой, не внутри неё
        let label_col = col + ICON_W.saturating_sub(icon.label.len()) / 2;
        draw_text_at(row + ICON_H, label_col, icon.label, Color::LightGray);
    }

    draw_taskbar(tasks, focused);
}

// Панель задач - геометрия кнопок (в пикселях). "CLIFF" слева, следом кнопка
// на каждый слот tasks[] по порядку (даже пустые - позиция кнопки не
// зависит от того, что открыто в других слотах, иначе кнопки "прыгали" бы
// при закрытии более ранней задачи). Всё это должно влезать даже в
// классический 320x200 (grid_cols()=53) - см. проверку при подборе чисел
// ниже TASKBAR_HOME_W
const TASKBAR_HOME_W: usize = 7 * CELL_W;
const TASKBAR_TASKS_X0: usize = 8 * CELL_W;
const TASKBAR_BUTTON_W: usize = 7 * CELL_W;
const TASKBAR_BUTTON_GAP: usize = CELL_W;
const TASKBAR_BUTTON_STRIDE: usize = TASKBAR_BUTTON_W + TASKBAR_BUTTON_GAP;

fn taskbar_panel_y0() -> usize {
    vga::video_height() - (CELL_H + 4)
}

fn taskbar_button_rect(i: usize) -> (usize, usize) {
    let x0 = TASKBAR_TASKS_X0 + i * TASKBAR_BUTTON_STRIDE;
    (x0, x0 + TASKBAR_BUTTON_W)
}

/// true - клик по "CLIFF" слева на панели задач (см. run()) - сбрасывает
/// фокус на рабочий стол, НЕ закрывая открытые задачи (аналог "Показать
/// рабочий стол"/свернуть всё)
fn home_button_hit(px: i32, py: i32) -> bool {
    let y0 = taskbar_panel_y0() as i32;
    px >= 0 && px < TASKBAR_HOME_W as i32 && py >= y0 && py < vga::video_height() as i32
}

/// Индекс задачи, чья кнопка на панели задач под точкой курсора (мышь) -
/// только среди РЕАЛЬНО открытых (пустые слоты не кликабельны, хотя их
/// позиция зарезервирована - см. taskbar_button_rect)
fn taskbar_task_hit(tasks: &[Option<Task>; MAX_TASKS], px: i32, py: i32) -> Option<usize> {
    let y0 = taskbar_panel_y0() as i32;
    if py < y0 || py >= vga::video_height() as i32 {
        return None;
    }
    for (i, slot) in tasks.iter().enumerate() {
        if slot.is_some() {
            let (x0, x1) = taskbar_button_rect(i);
            if px >= x0 as i32 && px < x1 as i32 {
                return Some(i);
            }
        }
    }
    None
}

/// Панель внизу экрана - "CLIFF" слева (клик - вернуться на рабочий стол, не
/// закрывая открытые задачи), кнопка на каждую открытую задачу (клик -
/// переключить фокус на неё, см. Task/MAX_TASKS/run()), часы (RTC) справа
fn draw_taskbar(tasks: &[Option<Task>; MAX_TASKS], focused: Option<usize>) {
    let panel_h_px = CELL_H + 4;
    let y0 = vga::video_height() - panel_h_px;
    vga::draw_rect(0, vga::video_width() - 1, y0, vga::video_height() - 1, Color::DarkGray);

    let text_y = y0 + 2;

    let home_focused = focused.is_none();
    if home_focused {
        vga::draw_rect(0, TASKBAR_HOME_W - 1, y0 + 1, y0 + panel_h_px - 2, Color::Blue);
    }
    let home_color = if home_focused { Color::Yellow } else { Color::LightGray };
    font::draw_text(3, text_y, "CLIFF", home_color);

    for (i, slot) in tasks.iter().enumerate() {
        if let Some(task) = slot {
            let (x0, x1) = taskbar_button_rect(i);
            let focused_here = Some(i) == focused;
            let fill = if focused_here { Color::Blue } else { Color::Black };
            vga::draw_rect(x0, x1 - 2, y0 + 1, y0 + panel_h_px - 2, fill);
            let text_color = if focused_here { Color::Yellow } else { Color::White };
            font::draw_text(x0 + 2, text_y, ICONS[task.icon].label, text_color);
        }
    }

    let now = crate::drivers::rtc::now();
    let mut buf = [0u8; 5];
    let clock = format_hms(&now, &mut buf);
    let clock_x = vga::video_width() - clock.len() * CELL_W - 3;
    font::draw_text(clock_x, text_y, clock, Color::White);
}

/// "HH:MM:SS" в буфер фиксированного размера
fn format_hms<'a>(now: &crate::drivers::rtc::DateTime, buf: &'a mut [u8; 5]) -> &'a str {
    buf[0] = b'0' + now.hour / 10;
    buf[1] = b'0' + now.hour % 10;
    buf[2] = b':';
    buf[3] = b'0' + now.minute / 10;
    buf[4] = b'0' + now.minute % 10;
    core::str::from_utf8(buf).unwrap_or("--:--")
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

    let height = vga::video_height();

    for y in 0..height {
        // Положение по вертикали в [0, bands) как fixed-point (шаг 1/256)
        let pos = y * bands * 256 / height;
        let band = (pos / 256).min(bands - 1);
        let frac = pos % 256; // насколько близко к следующему цвету полосы

        for x in 0..vga::video_width() {
            let dither = (x * 41 + y * 23) % 256;
            let color = if dither < frac { colors[band + 1] } else { colors[band] };
            vga::set_pixel(x, y, color);
        }
    }

    // Широкий диагональный блик (парабола) через весь экран
    let cx: isize = 360;
    let cy: isize = -60;
    for x in 0..vga::video_width() {
        let dx = x as isize - cx;
        let y = cy + (dx * dx) / 280;
        if (0..vga::video_height() as isize).contains(&y) {
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

/// Пиктограмма приложения - настоящий рисунок из примитивов (прямоугольники/
/// линии/круг - см. draw_disc), а не текстовый символ. `x0,y0,w,h` - зона
/// внутри рамки иконки (см. draw_desktop), формы рисуются с запасом от края,
/// координаты подобраны вручную под típичный размер зоны (~46x24px)
fn draw_icon(kind: usize, x0: usize, y0: usize, w: usize, h: usize) {
    let cx = x0 + w / 2;
    let cy = y0 + h / 2;

    match kind {
        0 => {
            // CALC - корпус, экран, два ряда кнопок
            vga::draw_rect(x0 + 4, x0 + w - 5, y0 + 1, y0 + h - 2, Color::LightGray);
            vga::draw_rect(x0 + 7, x0 + w - 8, y0 + 3, y0 + 7, Color::LightGreen);
            vga::draw_rect(x0 + 7, x0 + 13, y0 + h - 7, y0 + h - 4, Color::White);
            vga::draw_rect(x0 + 16, x0 + 22, y0 + h - 7, y0 + h - 4, Color::White);
        }
        1 => {
            // TEXTZ - лист бумаги с тремя строками текста
            vga::draw_rect(x0 + 8, x0 + w - 9, y0 + 1, y0 + h - 2, Color::White);
            vga::draw_hline(x0 + 11, x0 + w - 12, y0 + 5, Color::DarkGray);
            vga::draw_hline(x0 + 11, x0 + w - 12, y0 + 9, Color::DarkGray);
            vga::draw_hline(x0 + 11, x0 + w - 15, y0 + 13, Color::DarkGray);
        }
        2 => {
            // REALX - угловые скобки "< >" (как в коде)
            vga::draw_hline(x0 + 6, x0 + 12, y0 + 3, Color::LightGreen);
            vga::draw_vline(x0 + 6, y0 + 3, y0 + h - 4, Color::LightGreen);
            vga::draw_hline(x0 + 6, x0 + 12, y0 + h - 4, Color::LightGreen);
            vga::draw_hline(x0 + w - 13, x0 + w - 7, y0 + 3, Color::LightGreen);
            vga::draw_vline(x0 + w - 7, y0 + 3, y0 + h - 4, Color::LightGreen);
            vga::draw_hline(x0 + w - 13, x0 + w - 7, y0 + h - 4, Color::LightGreen);
        }
        3 => {
            // CLOCK - циферблат с двумя стрелками
            let r = (h / 2).saturating_sub(1);
            draw_disc(cx, cy, r, Color::LightCyan);
            vga::draw_vline(cx, cy.saturating_sub(r.saturating_sub(2)), cy, Color::Black);
            vga::draw_hline(cx, cx + r.saturating_sub(3), cy, Color::Black);
        }
        4 => {
            // MY PC - монитор на подставке
            vga::draw_rect(x0 + 5, x0 + w - 6, y0 + 1, y0 + h - 6, Color::Blue);
            vga::draw_rect(x0 + w / 2 - 3, x0 + w / 2 + 2, y0 + h - 5, y0 + h - 3, Color::DarkGray);
            vga::draw_hline(x0 + w / 2 - 6, x0 + w / 2 + 5, y0 + h - 2, Color::DarkGray);
        }
        _ => {
            // PAINT - палитра художника с "дыркой" для пальца и мазками
            // краски по краю (квадраты, не одиночные пиксели - иначе почти
            // не видно на таком масштабе)
            let r = (h / 2).saturating_sub(1);
            draw_disc(cx, cy, r, Color::Brown);
            draw_disc(cx + r / 2, cy + r / 2, 2, Color::Black);

            vga::draw_rect(cx - r + 1, cx - r + 3, cy - 2, cy, Color::Red);
            vga::draw_rect(cx - 2, cx, cy - r + 1, cy - r + 3, Color::Yellow);
            vga::draw_rect(cx + r - 3, cx + r - 1, cy - 2, cy, Color::LightGreen);
            vga::draw_rect(cx - 2, cx, cy + r - 3, cy + r - 1, Color::Blue);
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
        Layer::Paint(paint) => draw_paint_window(row, col, w, h, paint),
    }
}

/// Paint - холст paint_visible_size(w, h) квадратов paint::CELL_PX пикселей
/// каждый (растёт вместе с окном - см. paint_visible_size), отрисованных
/// заново из состояния (paint.cell()) каждый кадр, плюс обводка-курсор
/// поверх. Позиция окна фиксирована, размер - нет (см. handle_top)
fn draw_paint_window(row: usize, col: usize, w: usize, h: usize, paint: &PaintApp) {
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

    let (visible_cols, visible_rows) = paint_visible_size(w, h);
    for cy in 0..visible_rows {
        for cx in 0..visible_cols {
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
    fill_interior(row, col, w, h, Color::Black);
}

fn fill_interior(row: usize, col: usize, w: usize, h: usize, color: Color) {
    if w < 3 || h < 3 { return; }
    let x1 = col * CELL_W + 1;
    let x2 = col * CELL_W + w * CELL_W - 2;
    let y1 = row * CELL_H + 1;
    let y2 = row * CELL_H + h * CELL_H - 2;
    vga::draw_rect(x1, x2, y1, y2, color);
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
