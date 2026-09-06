// © Realix > Command: Stopwatch
// (15.08.26) v0.1
// ================

// Подключение функций
use crate::drivers::keyboard;
use crate::drivers::pit;
use crate::drivers::vga::{self, Color};
use crate::utils;

// Интервал обновления счётчика (мс)
const UPDATE_INTERVAL_MS: u32 = 200;

/// Запуск секундомера, остановка по нажатию клавиши
pub fn run() {
    // Точка отсчёта
    let start: u32 = pit::get_uptime();

    // Значение и ширина отрисованного значения (для перерисовки)
    let mut shown_value: u32 = u32::MAX;
    let mut shown_len: usize = 0;

    vga::print_line("[+] Stopwatch started. (Press any key to stop)\n", Color::LightGray);
    vga::print_line("Elapsed time (seconds): ", Color::LightGray);

    loop {
        // Перерисовываем число, только когда оно изменилось
        let elapsed: u32 = pit::get_uptime() - start;
        if elapsed != shown_value {
            // Стираем символы прошлого числа
            for _ in 0..shown_len {
                vga::print_backspace();
            }

            // Вывод сколько прошло секунд
            let mut str_buffer: [u8; 10] = [0u8; 10];
            let elapsed_str: &str = utils::u32_to_dec_str(elapsed, &mut str_buffer);
            vga::print_line(elapsed_str, Color::White);

            // Обновление переменных
            shown_len = elapsed_str.len();
            shown_value = elapsed;
        }

        // Выход по любой нажатой клавише (отпускания игнорируем)
        if let Some(scancode) = keyboard::queue_pop() {
            if scancode & keyboard::SCANCODE_RELEASE == 0 {
                break;
            }
        }

        pit::sleep(UPDATE_INTERVAL_MS);
    }
}
