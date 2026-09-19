// © Realix > Build (Config)
// (16.08.26) v0.12
// ================
// ❗️ Собирается на хост-машине во время сборки, можно использовать std
//    Также текущая задача, создать копию конфига, config.rs, для kernel32

use std::{env, fs, path::Path};

/// Основной код (Создание config.rs)
fn main() {
    // 1] Находим путь до shared/config.asm
    //   - Manifest: путь до этого файла (build.rs)
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();

    //   - Строим путь до sharedconfig.asm
    let asm_path = Path::new(&manifest).join("source/shared/config.asm");

    // Настройка кэширования результата (При изменении - пересборка)
    println!("cargo:rerun-if-changed={}", asm_path.display());

    // Читаем конфиг в одну строку (Если файла нет — ошибка)
    let content = fs::read_to_string(&asm_path)
        .unwrap_or_else(|e| panic!("[!] Не удалось прочитать {}: {}", asm_path.display(), e));

    // Переменная-накопление текста config.rs
    let mut out = String::new();

    // 2] Парсинг config.asm
    for line in content.lines() {
        // Отрезаем комментарии (Берём содержимое до значка комментария)
        let line = line.split(';').next().unwrap().trim();

        // Пропуск пустой строки (Или строки-комментария, ставшей пустой)
        if line.is_empty() { continue; }

        // Разбиваем строку на слова по пробелам
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Пропускаем, если не 3 "слова" в строке (Поддержка пока только таких форматов)
        if parts.len() != 3 {
            continue;
        }

        // 2.1] Пытаемся распознать один из двух знакомых форматов константы:
        //  - ИМЯ equ ЗНАЧЕНИЕ     (например: STACK_SIZE equ 0x4000)
        //  - %define ИМЯ ЗНАЧЕНИЕ (например: %define OS_VERSION 'v0.12')
        let (name, value) = if parts[1] == "equ" {
            (parts[0], parts[2])
        } else if parts[0] == "%define" {
            (parts[1], parts[2])
        } else {
            // Здесь остаётся, что пока что сознательно не поддерживается:
            //  - "%define ENTER 0x0D, 0x0A"          (два значения через запятую — 4 слова)
            //  - "KERNEL32_PHYS_ADDR equ (A*16 + B)" (выражение - не одно слово по пробелам)
            //  - "%ifndef CONFIG_ASM"                (директива препроцессора, не константа)
            //  - "%define CONFIG_ASM"                (define без значения — 2 слова)
            // Всё это молча пропускаем...
            continue;
        };

        // 2.2] Определяем, тип константы (число или строка), и генерируем Rust-код
        //
        // В config.asm встречаются два вида значений:
        //  - 0x4000: число (шестнадцатеричное или обычное)
        //  - 'v0.12': строка в одинарных кавычках
        if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
            // - Строка
            // > value.len() >= 2 нужно, чтобы не словить панику на одиночной кавычке (без пары),
            //                    хотя в валидном config.asm такого быть не должно...

            // Срезаем кавычки по краям: 'v0.12' -> v0.12
            let string_value = &value[1..value.len() - 1];

            // Генерируем Rust-строку
            // > {:?} автоматически оборачивает строку в кавычки и экранирует спецсимволы
            out += &format!("pub const {}: &str = {:?};\n", name, string_value);
        } else {
            // - Число (Парсим hex и dec)
            let parsed = if let Some(hex) = value.strip_prefix("0x") {
                u64::from_str_radix(hex, 16).ok()
            } else {
                value.parse::<u64>().ok()
            };

            match parsed {
                // Число успешно распозналось — генерируем числовую константу
                Some(val) => out += &format!("pub const {}: usize = {};\n", name, val),

                // Число не распозналось, выдаём предупреждение
                None => {
                    println!(
                        "cargo:warning=config.asm: пропущена константа {} — значение '{}' не распознано как число или строка",
                        name, value
                    );
                }
            }
        }
    }

    // 3] Записываем результат (Во временную папку сборки - target)
    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(Path::new(&out_dir).join("config.rs"), out).unwrap();
}