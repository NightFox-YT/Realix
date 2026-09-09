// © Realix > Command: Cliff (tiny desktop environment)
// (09.09.26) v0.1
// ================
// ❗️ Зависимости: x86::realmode_video (переход в VGA 320x200x256),
//    drivers::font (отрисовка текста в графическом режиме)
// ❗️ Мышь не поддерживается - нет PS/2-драйвера мыши (отдельная задача:
//    новый IRQ12-драйвер, разбор пакетов, отрисовка курсора, hit-testing).
//    Навигация - только клавиатурой: стрелки двигают окно, Enter/Space
//    открывают иконку, Escape закрывает окно/выходит из Cliff.
// ❗️ Перерисовка всего экрана на каждое событие, без разбиения на "грязные"
//    прямоугольники - при 320x200 и вводе по одной клавише это не заметно,
//    но полноценный менеджер окон захотел бы более умную инвалидацию.

// Подключение функций
use crate::config::{TEXT_MODE_80x25, VIDEO_MODE_320x200};
use crate::drivers::font;
use crate::drivers::keyboard::{self, Key};
use crate::drivers::vga::{self, Color};
use crate::x86::realmode_video;

// Иконка "Калькулятор" на рабочем столе
const ICON_X: usize = 20;
const ICON_Y: usize = 20;
const ICON_W: usize = 48;
const ICON_H: usize = 36;

// Окно калькулятора
const WINDOW_W: usize = 150;
const WINDOW_H: usize = 70;
const TITLEBAR_H: usize = 10;
const WINDOW_MOVE_STEP: usize = 4;

// Максимальная длина вводимого выражения (напр. "999999999+999999999")
const CALC_INPUT_CAP: usize = 24;

/// Текущий экран Cliff
enum Screen {
    Desktop,
    Calculator,
}

/// Состояние калькулятора (сохраняется между перерисовками окна)
struct CalcState {
    input: [u8; CALC_INPUT_CAP],
    input_len: usize,
    result: Option<i64>,
    error: bool,
}

impl CalcState {
    fn new() -> Self {
        CalcState { input: [0; CALC_INPUT_CAP], input_len: 0, result: None, error: false }
    }

    fn input_str(&self) -> &str {
        core::str::from_utf8(&self.input[..self.input_len]).unwrap_or("")
    }

    /// Сброс на новый ввод (после результата/ошибки, если пришла новая цифра)
    fn clear(&mut self) {
        self.input_len = 0;
        self.result = None;
        self.error = false;
    }

    fn push(&mut self, byte: u8) {
        // Начатый заново ввод после показанного результата/ошибки
        if self.result.is_some() || self.error {
            self.clear();
        }
        if self.input_len < CALC_INPUT_CAP {
            self.input[self.input_len] = byte;
            self.input_len += 1;
        }
    }

    fn backspace(&mut self) {
        self.result = None;
        self.error = false;
        if self.input_len > 0 {
            self.input_len -= 1;
        }
    }

    fn evaluate(&mut self) {
        match evaluate_expression(&self.input[..self.input_len]) {
            Ok(value) => { self.result = Some(value); self.error = false; }
            Err(()) => { self.result = None; self.error = true; }
        }
    }
}

/// Выполняет cliff: переключает VGA в 320x200x256 и показывает рабочий стол
/// с одной иконкой ("Калькулятор"), открываемой по Enter/Space; окно можно
/// двигать стрелками и закрыть по Escape. Второй Escape (на рабочем столе)
/// выходит из Cliff и возвращает текстовый режим
pub fn run() {
    vga::print_line(
        "Starting Cliff... (arrows move, Enter/Space opens, Escape closes/exits)\n",
        Color::LightGray,
    );

    realmode_video::set_mode(VIDEO_MODE_320x200 as u8);

    let mut screen = Screen::Desktop;
    let mut win_x: usize = (vga::VGA_VIDEO_WIDTH - WINDOW_W) / 2;
    let mut win_y: usize = 50;
    let mut calc = CalcState::new();

    draw_desktop();

    loop {
        match screen {
            Screen::Desktop => match keyboard::read_key() {
                Key::Escape => break,
                Key::Char(b'\n') | Key::Char(b' ') => {
                    screen = Screen::Calculator;
                    draw_window(win_x, win_y, &calc);
                }
                _ => {} // Стрелки - нет смысла двигать выбор, иконка одна
            },

            Screen::Calculator => match keyboard::read_key() {
                Key::Escape => {
                    screen = Screen::Desktop;
                    draw_desktop();
                }
                Key::Up => {
                    win_y = win_y.saturating_sub(WINDOW_MOVE_STEP);
                    redraw_window(win_x, win_y, &calc);
                }
                Key::Down => {
                    win_y = (win_y + WINDOW_MOVE_STEP).min(vga::VGA_VIDEO_HEIGHT - WINDOW_H);
                    redraw_window(win_x, win_y, &calc);
                }
                Key::Left => {
                    win_x = win_x.saturating_sub(WINDOW_MOVE_STEP);
                    redraw_window(win_x, win_y, &calc);
                }
                Key::Right => {
                    win_x = (win_x + WINDOW_MOVE_STEP).min(vga::VGA_VIDEO_WIDTH - WINDOW_W);
                    redraw_window(win_x, win_y, &calc);
                }
                Key::Char(b'\x08') => {
                    calc.backspace();
                    draw_window(win_x, win_y, &calc);
                }
                Key::Char(b'\n') | Key::Char(b'=') => {
                    calc.evaluate();
                    draw_window(win_x, win_y, &calc);
                }
                Key::Char(byte) if is_calc_input_char(byte) => {
                    calc.push(byte);
                    draw_window(win_x, win_y, &calc);
                }
                _ => {}
            },
        }
    }

    realmode_video::set_mode(TEXT_MODE_80x25 as u8);
    vga::init(0);
    vga::clear_screen();
    vga::print_line("Left Cliff.\n", Color::LightGray);
}

/// Символы, принимаемые калькулятором как ввод выражения
fn is_calc_input_char(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'*' | b'/')
}

/// Движение окна - т.к. кадр перерисовывается целиком, старую позицию
/// стирать отдельно не нужно: сперва стол (и вместе с ним всё "под" окном)
fn redraw_window(win_x: usize, win_y: usize, calc: &CalcState) {
    draw_desktop();
    draw_window(win_x, win_y, calc);
}

/// Рабочий стол: фон + иконка "Калькулятор" (всегда выделена - иконка одна)
fn draw_desktop() {
    vga::fill_screen(Color::Blue);

    vga::draw_rect(ICON_X, ICON_X + ICON_W, ICON_Y, ICON_Y + ICON_H, Color::LightGray);
    draw_border(ICON_X, ICON_Y, ICON_X + ICON_W, ICON_Y + ICON_H, Color::Yellow);

    let label = "CALC";
    let label_w = font::text_width(label.len());
    font::draw_text(
        ICON_X + (ICON_W - label_w) / 2,
        ICON_Y + (ICON_H - font::GLYPH_HEIGHT) / 2,
        label,
        Color::Black,
    );
}

/// Окно калькулятора: рамка + заголовок + поле ввода/результата
fn draw_window(win_x: usize, win_y: usize, calc: &CalcState) {
    let win_x2 = win_x + WINDOW_W;
    let win_y2 = win_y + WINDOW_H;

    // Рамка
    draw_border(win_x, win_y, win_x2, win_y2, Color::Black);

    // Заголовок
    vga::draw_rect(win_x + 1, win_x2 - 1, win_y + 1, win_y + TITLEBAR_H, Color::Blue);
    font::draw_text(win_x + 3, win_y + 2, "CALC", Color::White);
    font::draw_text(win_x2 - font::CHAR_ADVANCE - 2, win_y + 2, "X", Color::White);

    // Содержимое
    vga::draw_rect(win_x + 1, win_x2 - 1, win_y + TITLEBAR_H + 1, win_y2 - 1, Color::White);

    let text_x = win_x + 4;
    font::draw_text(text_x, win_y + TITLEBAR_H + 4, calc.input_str(), Color::Black);

    if calc.error {
        font::draw_text(text_x, win_y + TITLEBAR_H + 4 + font::GLYPH_HEIGHT + 2, "ERR", Color::Red);
    } else if let Some(result) = calc.result {
        let mut buf: [u8; 12] = [0; 12];
        let text = format_result(result, &mut buf);
        font::draw_text(text_x, win_y + TITLEBAR_H + 4 + font::GLYPH_HEIGHT + 2, text, Color::Blue);
    }
}

/// Отрисовка прямоугольной рамки (только контур, без заливки)
fn draw_border(x1: usize, y1: usize, x2: usize, y2: usize, color: Color) {
    vga::draw_hline(x1, x2, y1, color);
    vga::draw_hline(x1, x2, y2, color);
    vga::draw_vline(x1, y1, y2, color);
    vga::draw_vline(x2, y1, y2, color);
}

/// Форматирование результата вычисления в вид "=N" или "=-N" в буфер
fn format_result<'a>(value: i64, buf: &'a mut [u8; 12]) -> &'a str {
    buf[0] = b'=';

    let (negative, magnitude) = if value < 0 { (true, (-value) as u32) } else { (false, value as u32) };
    let mut num_buf: [u8; 10] = [0u8; 10];
    let num_str = crate::utils::u32_to_dec_str(magnitude, &mut num_buf);

    let mut pos = 1;
    if negative {
        buf[pos] = b'-';
        pos += 1;
    }
    for &b in num_str.as_bytes() {
        if pos >= buf.len() { break; }
        buf[pos] = b;
        pos += 1;
    }

    core::str::from_utf8(&buf[..pos]).unwrap_or("=?")
}

/// Разбор и вычисление выражения вида "ЧИСЛО ОПЕРАТОР ЧИСЛО" (без пробелов,
/// один оператор, только целые числа) - та же идея, что и у команды calc,
/// но без пробелов между токенами (ввод идёт по одному символу)
fn evaluate_expression(input: &[u8]) -> Result<i64, ()> {
    let op_pos = input.iter().position(|&b| !b.is_ascii_digit()).ok_or(())?;
    if op_pos == 0 {
        return Err(()); // Нет числа перед оператором
    }

    let (a_bytes, rest) = input.split_at(op_pos);
    let op = rest[0];
    let b_bytes = &rest[1..];

    if b_bytes.is_empty() || !b_bytes.iter().all(u8::is_ascii_digit) {
        return Err(());
    }

    let a = parse_dec(a_bytes)?;
    let b = parse_dec(b_bytes)?;

    let result = match op {
        b'+' => a.checked_add(b),
        b'-' => a.checked_sub(b),
        b'*' => a.checked_mul(b),
        b'/' => if b == 0 { None } else { Some(a / b) },
        _ => None,
    };

    result.filter(|v| (-(u32::MAX as i64)..=u32::MAX as i64).contains(v)).ok_or(())
}

/// Разбор беззнаковой десятичной последовательности байт в i64
fn parse_dec(bytes: &[u8]) -> Result<i64, ()> {
    if bytes.is_empty() {
        return Err(());
    }

    let mut value: i64 = 0;
    for &b in bytes {
        value = value.checked_mul(10).ok_or(())?.checked_add((b - b'0') as i64).ok_or(())?;
    }
    Ok(value)
}
