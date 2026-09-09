// © Realix > Cliff: приложение "Калькулятор"
// ================

// Максимальная длина вводимого выражения (напр. "999999999+999999999")
const CALC_INPUT_CAP: usize = 24;

/// Размер окна калькулятора (знакоместа) - по умолчанию и границы для
/// изменения размера через Shift+стрелки (см. cliff::apply_resize)
pub const DEFAULT_W: usize = 20;
pub const DEFAULT_H: usize = 5;
pub const MIN_W: usize = 16;
pub const MAX_W: usize = 40;
pub const MIN_H: usize = 5;
pub const MAX_H: usize = 10;

/// Состояние калькулятора (сохраняется между перерисовками окна)
pub struct CalcApp {
    input: [u8; CALC_INPUT_CAP],
    input_len: usize,
    result: Option<i64>,
    error: bool,
}

impl CalcApp {
    pub fn new() -> Self {
        CalcApp { input: [0; CALC_INPUT_CAP], input_len: 0, result: None, error: false }
    }

    pub fn input_str(&self) -> &str {
        core::str::from_utf8(&self.input[..self.input_len]).unwrap_or("")
    }

    pub fn error(&self) -> bool { self.error }
    pub fn result(&self) -> Option<i64> { self.result }

    /// Сброс на новый ввод (после результата/ошибки, если пришла новая цифра)
    fn clear(&mut self) {
        self.input_len = 0;
        self.result = None;
        self.error = false;
    }

    pub fn push(&mut self, byte: u8) {
        if !is_calc_input_char(byte) {
            return;
        }
        // Начатый заново ввод после показанного результата/ошибки
        if self.result.is_some() || self.error {
            self.clear();
        }
        if self.input_len < CALC_INPUT_CAP {
            self.input[self.input_len] = byte;
            self.input_len += 1;
        }
    }

    pub fn backspace(&mut self) {
        self.result = None;
        self.error = false;
        if self.input_len > 0 {
            self.input_len -= 1;
        }
    }

    pub fn evaluate(&mut self) {
        match evaluate_expression(&self.input[..self.input_len]) {
            Ok(value) => { self.result = Some(value); self.error = false; }
            Err(()) => { self.result = None; self.error = true; }
        }
    }
}

/// Символы, принимаемые калькулятором как ввод выражения
fn is_calc_input_char(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'*' | b'/')
}

/// Форматирование результата вычисления в вид "=N" или "=-N" в буфер
pub fn format_result<'a>(value: i64, buf: &'a mut [u8; 12]) -> &'a str {
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
