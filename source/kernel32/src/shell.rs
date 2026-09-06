// © Realix > Shell
// ø @liquifield
// (26.07.26) v0.1
// ================

// Импорт функций
use crate::commands;
use crate::drivers::keyboard::{self, read_key};
use crate::drivers::pit;
use crate::drivers::speaker;
use crate::drivers::vga::{self, Color};
use crate::utils;

// Константы
pub const INPUT_MAX: usize = 64;
const HISTORY_SIZE: usize = 16;
const HISTORY_SLOT_SIZE: usize = INPUT_MAX + 1;
const PROMPT: &str = "Realix >> ";

// История команд (Кольцевой буфер)
struct History {
    data: [[u8; HISTORY_SLOT_SIZE]; HISTORY_SIZE],
    browse: usize, // Позиция навигации (0 - текущая строка)
    next: usize,   // Индекс слота след. записи
    count: usize,  // Кол-во сохранённых команд
}

impl History {
    /// Создание экземпляра истории
    fn new() -> Self {
        History {
            data: [[0; HISTORY_SLOT_SIZE]; HISTORY_SIZE],
            browse: 0, next: 0, count: 0,
        }
    }

    /// Получение строки записи истории по позиции
    fn get(&self, browse: usize) -> &str {
        let slot: usize = (self.next + HISTORY_SIZE - browse) % HISTORY_SIZE;
        self.slot_str(slot)
    }

    /// Парсинг строки, хранящаяся в слоте (до нуль-терминатора)
    fn slot_str(&self, slot: usize) -> &str {
        let record: &[u8] = &self.data[slot];
        let end: usize = record.iter().position(|&b| b == 0).unwrap_or(record.len());
        core::str::from_utf8(&record[..end]).unwrap_or("")
    }

    /// Добавление непустой строки в историю (дубликат последней команды не сохраняется)
    fn add(&mut self, line: &str) {
        // Проверка на пустую строку
        if line.is_empty() { return; }

        // Проверка на то, что последняя запись не является дубликатом текущей
        if self.count > 0 {
            let last_slot: usize = (self.next + HISTORY_SIZE - 1) % HISTORY_SIZE;
            if self.slot_str(last_slot) == line {
                return;
            }
        }

        // Запись строки в слот (С преждевременной инициализацией с 0 значением)
        let bytes: &[u8] = line.as_bytes();
        let len: usize = bytes.len().min(HISTORY_SLOT_SIZE - 1);

        let slot: &mut [u8; HISTORY_SLOT_SIZE] = &mut self.data[self.next];
        slot.fill(0);
        slot[..len].copy_from_slice(&bytes[..len]);

        // Обновляем переменные истории
        self.next = (self.next + 1) % HISTORY_SIZE;
        self.count = (self.count + 1).min(HISTORY_SIZE);
    }

    /// Список записей истории от старой к новой, с номерами (команда F7)
    fn list(&self) {
        for i in 1..=self.count {
            let browse: usize = self.count - i + 1;
            let line: &str = self.get(browse);
            let mut num_buf: [u8; 10] = [0u8; 10];

            vga::print_line(utils::u32_to_dec_str(i as u32, &mut num_buf), Color::White);
            vga::print_line(": ", Color::LightGray);
            vga::print_line(line, Color::LightGray);
            vga::print_new_line();
        }
    }
}


/// Функция выполнения команды
/// ! Гарантируется, что переданный указатель содержит валидную строку
fn execute(input: &str) {
    // Форматируем введённую строку
    let input: &str = input.trim();

    // Имя команды — до первого пробела, остальное — аргументы
    let name_end: usize = input.find(' ').unwrap_or(input.len());
    let (name, args) = input.split_at(name_end);

    match name {
        "" => {}
        "help" => { commands::help::show(); }
        "clear" | "cls" => { vga::clear_screen(); }
        "reboot" => { commands::reboot::run() }
        "shutdown" => { commands::shutdown::run() }
        "echo" => { commands::echo::run(args) }
        "uptime" => {
            let mut str_buffer: [u8; 10] = [0u8; 10];

            vga::print_line("Uptime (seconds): ", Color::LightGray);
            vga::print_line(
                utils::u32_to_dec_str(pit::get_uptime(), &mut str_buffer),
                Color::LightGray);
            vga::print_new_line();
        }
        "meminfo" => { commands::meminfo::show(); }
        "matrix" => {
            vga::print_line("Entering Matrix... (Press any key to exit)\n", Color::Green);
            commands::matrix::run();
        }
        "stopwatch" => { commands::stopwatch::run(); }
        "nova" => {
            unsafe { commands::nova_ai::BC(args); }
        }
        _ => {
            vga::print_line("[!] Unknown command. Type 'help' for list of commands.\n", Color::Red);
        }
    }
}


/// CLI: Основной цикл
pub fn run() {
    vga::print_line("Type 'help' for list of commands.\n\n", Color::LightGray);
    let mut history: History = History::new();

    loop {
        vga::print_line(PROMPT, Color::Green);
        let input_array: [u8; HISTORY_SLOT_SIZE] = read_line(&mut history);

        // Пропускаем пустой ввод
        if input_array[0] == 0 {
            continue;
        }

        // Обрезаем по нуль-терминатору
        let input_str: &str = core::str::from_utf8(&input_array).unwrap_or("");
        let end_idx: usize = input_str.find('\0').unwrap_or(input_str.len());
        execute(&input_str[..end_idx]);

        vga::print_new_line_if_needed();
    }
}


/// Замена видимой строки ввода строкой `text` (стирание + перерисовка).
fn replace_input(buffer: &mut [u8; HISTORY_SLOT_SIZE], pos: &mut usize, text: &str) {
    // Стираем видимую часть текущей строки
    for _ in 0..*pos {
        vga::print_backspace();
    }

    // Печатаем новую строку и сохраняем её в буфер
    *pos = 0;
    for &byte in text.as_bytes().iter().take(INPUT_MAX) {
        buffer[*pos] = byte;
        *pos += 1;
        vga::print_char(byte, Color::LightGray);
    }
    buffer[*pos] = 0;
}


/// CLI: Чтение строки
fn read_line(history: &mut History) -> [u8; HISTORY_SLOT_SIZE] {
    let mut buffer: [u8; HISTORY_SLOT_SIZE] = [0u8; HISTORY_SLOT_SIZE];
    let mut pos: usize = 0;
    history.browse = 0;

    loop {
        match read_key() {
            keyboard::Key::Char(b'\n') => {
                vga::print_new_line();
                buffer[pos] = 0;

                let line: &str = core::str::from_utf8(&buffer[..pos]).unwrap_or("");
                history.add(line);

                return buffer;
            }
            keyboard::Key::Char(b'\x08') => {
                if pos > 0 {
                    pos -= 1;
                    buffer[pos] = 0;
                    vga::print_backspace();
                }
            }
            keyboard::Key::Escape => {
                replace_input(&mut buffer, &mut pos, "");
            }
            keyboard::Key::Up => {
                // Уже на самой старой записи
                if history.browse >= history.count {
                    continue;
                }

                history.browse += 1;
                let line: &str = history.get(history.browse);
                replace_input(&mut buffer, &mut pos, line);
            }
            keyboard::Key::Down => {
                // Уже на текущей строке (0)
                if history.browse == 0 {
                    continue;
                }

                history.browse -= 1;

                if history.browse == 0 {
                    replace_input(&mut buffer, &mut pos, "");
                } else {
                    let line: &str = history.get(history.browse);
                    replace_input(&mut buffer, &mut pos, line);
                }
            }
            keyboard::Key::F7 => {
                vga::print_new_line();
                history.list();

                // Заново показываем промпт и уже набранную строку
                vga::print_line(PROMPT, Color::Green);
                let line: &str = core::str::from_utf8(&buffer[..pos]).unwrap_or("");
                vga::print_line(line, Color::LightGray);
            }
            keyboard::Key::Char(byte) if pos < INPUT_MAX && (0x20..=0x7E).contains(&byte) => {
                buffer[pos] = byte;
                pos += 1;
                vga::print_char(byte, Color::LightGray);
            }
            // Буфер строки заполнен (pos == INPUT_MAX) — сигнализируем об этом,
            // а не просто молча игнорируем ввод
            keyboard::Key::Char(byte) if (0x20..=0x7E).contains(&byte) => {
                speaker::beep_short();
            }
            _ => {}
        }
    }
}
