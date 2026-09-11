// © Realix > Driver: VGA
// ø Inspired by @liquifield
// (24.08.26) v0.12
// ================
// ! Не вызывать из IRQ прерываний (Гонка данных на константах)

// Подключение функций
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use crate::utils::{self, outb};

// Константы
const VGA_TEXT_BUFFER: *mut u8 = 0xB8000 as *mut u8;
pub const VGA_TEXT_WIDTH: usize = 80;
pub const VGA_TEXT_HEIGHT: usize = 25;
pub const VGA_VIDEO_WIDTH: usize = 320;
pub const VGA_VIDEO_HEIGHT: usize = 200;
const EMPTY_CELL: u16 = (0x0F << 8) | b' ' as u16;  // Пробел + 0x0F (белый на чёрном)

// Регистры CRT-контроллера для управления аппаратным курсором
const VGA_CRTC_INDEX: u16 = 0x3D4;  // Индексный порт
const VGA_CRTC_DATA:  u16 = 0x3D5;  // Порт данных
const VGA_CURSOR_HIGH: u8 = 0x0E;   // Регистр старшего байта позиции курсора
const VGA_CURSOR_LOW:  u8 = 0x0F;   // Регистр младшего байта позиции курсора

// Позиция курсора
static CURSOR_ROW: AtomicUsize = AtomicUsize::new(0);
static CURSOR_COL: AtomicUsize = AtomicUsize::new(0);

// Верхняя граница прокрутки/очистки (0 по умолчанию - обычное поведение).
// Позволяет приложению зарезервировать верхние строки (напр. заголовок
// Cliff::terminax) так, чтобы text_clear_screen/scroll_up их не трогали -
// см. set_scroll_top ниже
static SCROLL_TOP: AtomicUsize = AtomicUsize::new(0);

/// Логическая ширина/высота холста, которым оперируют set_pixel/fill_screen
/// и вся раскладка cliff_gfx (grid_cols/grid_rows и т.п.) - всегда обычный
/// VGA mode 13h (320x200, фреймбуфер 0xA0000 построчно без отступов), это
/// единственный поддерживаемый графический режим
pub fn video_width() -> usize { VGA_VIDEO_WIDTH }
pub fn video_height() -> usize { VGA_VIDEO_HEIGHT }

// Таблица цветов
#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

/// Общая структура для одноименных функций в видеорежиме и текстовом режиме
pub struct VideoOps {
    pub print_line: fn(line: &str, color: Color),
    pub print_char: fn(char_byte: u8, color: Color),
    pub clear_screen: fn(),
}


/// Инициализация OPS, используя структуру VideoOps
pub fn init(videomode: u16) {
    unsafe {
        OPS = match videomode {
            0 => VideoOps {
                print_line: text_print_line,
                print_char: text_print_char,
                clear_screen: text_clear_screen,
            },

            // ! Пока что в видеорежиме нет таких обработчиков, оставляем заглушки
            1 => VideoOps {
                print_line: text_print_line,
                print_char: text_print_char,
                clear_screen: text_clear_screen,
            },
            _ => panic!("unsupported video mode"),
        };
    }
}


// Основной интерфейс (Дефолтный)
static mut OPS: VideoOps = VideoOps {
    print_line: text_print_line,
    print_char: text_print_char,
    clear_screen: text_clear_screen,
};

// Публичный интерфейс модуля
pub fn clear_screen() { unsafe { (OPS.clear_screen)() } }
pub fn print_char(char_byte: u8, color: Color) { unsafe { (OPS.print_char)(char_byte, color) } }
pub fn print_line(line: &str, color: Color) { unsafe { (OPS.print_line)(line, color) } }


/// Отдельно от OPS — эта функция вообще не должна дёргаться в текстовом режиме
pub fn set_pixel(x: usize, y: usize, color: Color) {
    if x >= video_width() || y >= video_height() {
        return;
    }
    write_pixel_unchecked(x, y, color);
}

fn write_pixel_unchecked(x: usize, y: usize, color: Color) {
    unsafe {
        let offset = y * VGA_VIDEO_WIDTH + x;
        (0xA0000 as *mut u8).add(offset).write_volatile(color as u8);
    }
}

/// Заливка всего экрана одним цветом (VGA Video)
pub fn fill_screen(color: Color) {
    for y in 0..video_height() {
        for x in 0..video_width() {
            set_pixel(x, y, color);
        }
    }
}


/// Отрисовка горизонтальной линии (VGA Video)
pub fn draw_hline(x1: usize, x2: usize, y: usize, color: Color) {
    for curr_x in x1..=x2 {
        set_pixel(curr_x, y, color);
    }
}

/// Отрисовка вертикальной линии (VGA Video)
pub fn draw_vline(x: usize, y1: usize, y2: usize, color: Color) {
    for curr_y in y1..=y2 {
        set_pixel(x, curr_y, color);
    }
}

/// Отрисовка вертикальной линии (VGA Video)
pub fn draw_rect(x1: usize, x2: usize, y1: usize, y2: usize, color: Color) {
    // Рисуем циклично горизонтальные строки
    for curr_y in y1..=y2 {
        draw_hline(x1, x2, curr_y, color);
    }
}
    

/// Заполнение `count` ячеек экрана пустой ячейкой, начиная с `start_cell`
fn clear_cells(start_cell: usize, count: usize) {
    let cells: *mut u16 = VGA_TEXT_BUFFER as *mut u16;

    for i in start_cell..start_cell + count {
        unsafe { cells.add(i).write_volatile(EMPTY_CELL); }
    }
}


/// Очистка экрана (от SCROLL_TOP и ниже - см. set_scroll_top) и сброс
/// курсора в верхний левый угол ОБЛАСТИ (не обязательно строку 0)
pub fn text_clear_screen() {
    let top: usize = SCROLL_TOP.load(Relaxed);

    clear_cells(top * VGA_TEXT_WIDTH, (VGA_TEXT_HEIGHT - top) * VGA_TEXT_WIDTH);

    CURSOR_ROW.store(top, Relaxed);
    CURSOR_COL.store(0, Relaxed);
    update_cursor();
}


/// Резервирует верхние `row` строк экрана от очистки/прокрутки (0 -
/// обычное поведение на весь экран) - используется приложениями вроде
/// Cliff::terminax, которым нужна строка заголовка, не участвующая в
/// прокрутке содержимого. Сбрасывает курсор в начало новой рабочей области
pub fn set_scroll_top(row: usize) {
    SCROLL_TOP.store(row.min(VGA_TEXT_HEIGHT - 1), Relaxed);
    CURSOR_ROW.store(row.min(VGA_TEXT_HEIGHT - 1), Relaxed);
    CURSOR_COL.store(0, Relaxed);
    update_cursor();
}


/// Поднять все строки рабочей области (см. SCROLL_TOP) на `n` позиций
fn scroll_up(lines_count: usize) {
    let top: usize = SCROLL_TOP.load(Relaxed);
    let usable_height: usize = VGA_TEXT_HEIGHT - top;

    // Если кол-во строк для прокрутки больше чем высота рабочей области
    if lines_count >= usable_height {
        text_clear_screen();
        return;
    }

    // Копируем строки `top+n`..HEIGHT в начало рабочей области единым
    // блоком (memmove); строки выше `top` (если есть) не трогаем
    unsafe {
        core::ptr::copy(
            VGA_TEXT_BUFFER.add((top + lines_count) * VGA_TEXT_WIDTH * 2),
            VGA_TEXT_BUFFER.add(top * VGA_TEXT_WIDTH * 2),
            (usable_height - lines_count) * VGA_TEXT_WIDTH * 2
        );
    }

    // "Стираем" последние `n` строк рабочей области пробелами
    clear_cells((top + usable_height - lines_count) * VGA_TEXT_WIDTH, lines_count * VGA_TEXT_WIDTH);

    // Обновляем позицию курсора на `n` строк вверх (не выше `top`)
    let row: usize = CURSOR_ROW.load(Relaxed);
    CURSOR_ROW.store(row.saturating_sub(lines_count).max(top), Relaxed);
    update_cursor();
}


/// Перенос аппаратного курсора CRTC в текущую позицию
fn update_cursor() {
    // Вычисляем новую позицию
    let row: usize = CURSOR_ROW.load(Relaxed);
    let col: usize = CURSOR_COL.load(Relaxed);
    let pos: u16 = (row * VGA_TEXT_WIDTH + col) as u16;

    // Выбираем регистр CRTC и передаём младший, затем старший байт позиции курсора
    unsafe {
        outb(VGA_CRTC_INDEX, VGA_CURSOR_LOW);
        outb(VGA_CRTC_DATA, (pos & 0xFF) as u8);

        outb(VGA_CRTC_INDEX, VGA_CURSOR_HIGH);
        outb(VGA_CRTC_DATA, (pos >> 8) as u8);
    }
}


/// Вывод символа на экран (по принципам TTY)
pub fn text_print_char(char_byte: u8, color: Color) {
    match char_byte {
        b'\n' => {
            CURSOR_COL.store(0, Relaxed);
            CURSOR_ROW.fetch_add(1, Relaxed);
        }
        b'\r' => { CURSOR_COL.store(0, Relaxed); }
        _ => {
            let row: usize = CURSOR_ROW.load(Relaxed);
            let col: usize = CURSOR_COL.load(Relaxed);
            let offset: usize = (row * VGA_TEXT_WIDTH + col) * 2;
            unsafe {
                VGA_TEXT_BUFFER.add(offset).write_volatile(char_byte);
                VGA_TEXT_BUFFER.add(offset + 1).write_volatile(color as u8);
            }
            CURSOR_COL.fetch_add(1, Relaxed);
        }
    }

    // Перенос курсора на след. строку
    if CURSOR_COL.load(Relaxed) >= VGA_TEXT_WIDTH {
        CURSOR_COL.store(0, Relaxed);
        CURSOR_ROW.fetch_add(1, Relaxed);
    }

    // Если курсор выходит за нижнюю границу экрана
    while CURSOR_ROW.load(Relaxed) >= VGA_TEXT_HEIGHT {
        scroll_up(1);
    }

    update_cursor();
}

/// Перемещение аппаратного курсора в произвольную позицию, БЕЗ изменения
/// сохранённой позиции печати (CURSOR_ROW/COL) - для приложений типа Cliff,
/// рисующих напрямую через write_char_at и хотящих показать текстовый
/// курсор (напр. редактор TextZ/RealX IDE). Безопасно, т.к. такие приложения
/// не вызывают print_char/print_line - CURSOR_ROW/COL при выходе всё равно
/// сбрасываются в (0,0) через text_clear_screen()
pub fn set_cursor_pos(row: usize, col: usize) {
    let pos: u16 = (row * VGA_TEXT_WIDTH + col) as u16;
    unsafe {
        outb(VGA_CRTC_INDEX, VGA_CURSOR_LOW);
        outb(VGA_CRTC_DATA, (pos & 0xFF) as u8);
        outb(VGA_CRTC_INDEX, VGA_CURSOR_HIGH);
        outb(VGA_CRTC_DATA, (pos >> 8) as u8);
    }
}

/// Вывод символа в определённой позиции
pub fn write_char_at(row: usize, col: usize, char_byte: u8, color: Color) {
    // Проверка, что символ находитсья в пределах экрана
    if row >= VGA_TEXT_HEIGHT || col >= VGA_TEXT_WIDTH {
        return;
    }

    // Запись символа
    let offset: usize = (row * VGA_TEXT_WIDTH + col) * 2;
    unsafe {
        VGA_TEXT_BUFFER.add(offset).write_volatile(char_byte);
        VGA_TEXT_BUFFER.add(offset + 1).write_volatile(color as u8);
    }
}


/// Вывод строки на экран (по принципам TTY)
pub fn text_print_line(line: &str, color: Color) {
    for byte in line.bytes() {
        print_char(byte, color);
    }
}


/// Стирание последнего символа с переносом курсора назад
pub fn print_backspace() {
    // > Обновление позиции курсора (не выше SCROLL_TOP - см. set_scroll_top)
    let top: usize = SCROLL_TOP.load(Relaxed);
    if CURSOR_COL.load(Relaxed) > 0 {
        CURSOR_COL.fetch_sub(1, Relaxed);
    } else if CURSOR_ROW.load(Relaxed) > top {
        CURSOR_ROW.fetch_sub(1, Relaxed);
        CURSOR_COL.store(VGA_TEXT_WIDTH - 1, Relaxed);
    }

    // Замена последнего символа на пробел
    let row: usize = CURSOR_ROW.load(Relaxed);
    let col: usize = CURSOR_COL.load(Relaxed);
    let offset: usize = (row * VGA_TEXT_WIDTH + col) * 2;
    unsafe {
        VGA_TEXT_BUFFER.add(offset).write_volatile(b' ');
        VGA_TEXT_BUFFER.add(offset + 1).write_volatile(0x0F);
    }

    update_cursor();
}

/// Перевод строки
pub fn print_new_line() {
    print_char(b'\n', Color::White);
}

/// Перевод строки, если надо
pub fn print_new_line_if_needed() {
    if CURSOR_COL.load(Relaxed) != 0 {
        print_char(b'\n', Color::White);
    }
}

/// Вывод строки дампа регистра
pub fn print_reg_line(reg_label: &str, value: u32) {
    let mut buffer: [u8; 10] = [0u8; 10];

    print_line("> ", Color::LightGray);
    print_line(reg_label, Color::LightGray);
    print_line(" = ", Color::LightGray);
    print_line(utils::u32_to_hex_str(value, &mut buffer), Color::White);
    print_new_line();
}