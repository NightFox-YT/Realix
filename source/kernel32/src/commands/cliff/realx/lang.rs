// © Realix > Cliff: RealX - ультра-базовый язык программирования
// ================
// ❗️ Синтаксис нарочно похож на Python (переменные, print(...), if/while с
// операторами сравнения), но с явным `end` вместо отступов - настоящий
// Python-style off-side rule (отступы как границы блоков) требует полноценного
// токенайзера с отслеживанием уровня вложенности; `end` даёт тот же результат
// (читаемый, "по-питоновски" выглядящий код) при спокойно более простом
// построчном интерпретаторе - см. run() ниже
// ❗️ Ограничения (осознанно, "ультра-базовый" язык):
//    - Только целые числа (i64); строки - только литералы в print(), не
//      сохраняются в переменных
//    - До 16 переменных, имя до 8 символов
//    - if/while - ровно одно условие сравнения (== != < <= > >=), без
//      elif/else, без вложенных выражений сложнее (a + b * c) со скобками
//    - Нет пользовательских функций (def) - только последовательный код,
//      условия и циклы
//    - Защита от зависания: программа обрывается с ошибкой после
//      MAX_STEPS выполненных строк (напр. `while 1 == 1:` без выхода)

use super::super::editor::{Editor, LINE_LEN, MAX_LINES};

// Максимум переменных и длина имени переменной
const MAX_VARS: usize = 16;
const VAR_NAME_LEN: usize = 8;

// Максимум вложенности if/while
const MAX_BLOCK_DEPTH: usize = 6;

// Предел выполненных "шагов" (строк) - защита от бесконечного цикла
const MAX_STEPS: usize = 200_000;

#[derive(Clone, Copy)]
struct Var {
    name: [u8; VAR_NAME_LEN],
    name_len: u8,
    value: i64,
}

struct VarTable {
    vars: [Option<Var>; MAX_VARS],
}

impl VarTable {
    fn new() -> Self {
        VarTable { vars: [None; MAX_VARS] }
    }

    fn find(&self, name: &[u8]) -> Option<usize> {
        for i in 0..MAX_VARS {
            if let Some(v) = &self.vars[i] {
                if &v.name[..v.name_len as usize] == name {
                    return Some(i);
                }
            }
        }
        None
    }

    fn get(&self, name: &[u8]) -> Option<i64> {
        self.find(name).map(|i| self.vars[i].unwrap().value)
    }

    /// Ok(()) при успехе; Err(()) если имя слишком длинное или таблица полна
    fn set(&mut self, name: &[u8], value: i64) -> Result<(), ()> {
        if let Some(i) = self.find(name) {
            self.vars[i].as_mut().unwrap().value = value;
            return Ok(());
        }
        if name.is_empty() || name.len() > VAR_NAME_LEN {
            return Err(());
        }
        for i in 0..MAX_VARS {
            if self.vars[i].is_none() {
                let mut buf = [0u8; VAR_NAME_LEN];
                buf[..name.len()].copy_from_slice(name);
                self.vars[i] = Some(Var { name: buf, name_len: name.len() as u8, value });
                return Ok(());
            }
        }
        Err(())
    }
}

/// Построчный лексер - разбирает ОДНУ строку исходника за раз
struct Lexer<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Lexer { bytes, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ') | Some(b'\t')) {
            self.pos += 1;
        }
    }

    /// Читает идентификатор (ключевое слово или имя переменной)
    fn read_ident(&mut self) -> &'a [u8] {
        self.skip_ws();
        let start = self.pos;
        while matches!(self.peek(), Some(b) if b.is_ascii_alphanumeric() || b == b'_') {
            self.pos += 1;
        }
        &self.bytes[start..self.pos]
    }

    fn consume_byte(&mut self, byte: u8) -> bool {
        self.skip_ws();
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, s: &[u8]) -> bool {
        self.skip_ws();
        if self.bytes[self.pos..].starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }
}

fn parse_i64(bytes: &[u8]) -> Result<i64, ()> {
    if bytes.is_empty() {
        return Err(());
    }
    let mut value: i64 = 0;
    for &b in bytes {
        value = value.checked_mul(10).ok_or(())?.checked_add((b - b'0') as i64).ok_or(())?;
    }
    Ok(value)
}

fn parse_expr(lex: &mut Lexer, vars: &VarTable) -> Result<i64, ()> {
    let mut value = parse_term(lex, vars)?;
    loop {
        lex.skip_ws();
        match lex.peek() {
            Some(b'+') => { lex.pos += 1; value = value.checked_add(parse_term(lex, vars)?).ok_or(())?; }
            Some(b'-') => { lex.pos += 1; value = value.checked_sub(parse_term(lex, vars)?).ok_or(())?; }
            _ => break,
        }
    }
    Ok(value)
}

fn parse_term(lex: &mut Lexer, vars: &VarTable) -> Result<i64, ()> {
    let mut value = parse_factor(lex, vars)?;
    loop {
        lex.skip_ws();
        match lex.peek() {
            Some(b'*') => { lex.pos += 1; value = value.checked_mul(parse_factor(lex, vars)?).ok_or(())?; }
            Some(b'/') => {
                lex.pos += 1;
                let rhs = parse_factor(lex, vars)?;
                if rhs == 0 { return Err(()); }
                value /= rhs;
            }
            _ => break,
        }
    }
    Ok(value)
}

fn parse_factor(lex: &mut Lexer, vars: &VarTable) -> Result<i64, ()> {
    lex.skip_ws();
    match lex.peek() {
        Some(b'-') => { lex.pos += 1; Ok(-parse_factor(lex, vars)?) }
        Some(b'(') => {
            lex.pos += 1;
            let v = parse_expr(lex, vars)?;
            if !lex.consume_byte(b')') { return Err(()); }
            Ok(v)
        }
        Some(b) if b.is_ascii_digit() => {
            let start = lex.pos;
            while matches!(lex.peek(), Some(b) if b.is_ascii_digit()) { lex.pos += 1; }
            parse_i64(&lex.bytes[start..lex.pos])
        }
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
            let name = lex.read_ident();
            vars.get(name).ok_or(())
        }
        _ => Err(()),
    }
}

/// Разбор условия сравнения `<expr> <op> <expr>` - ровно один оператор
fn parse_condition(lex: &mut Lexer, vars: &VarTable) -> Result<bool, ()> {
    let lhs = parse_expr(lex, vars)?;
    lex.skip_ws();

    let cmp = if lex.consume_str(b"==") { 0 }
        else if lex.consume_str(b"!=") { 1 }
        else if lex.consume_str(b"<=") { 2 }
        else if lex.consume_str(b">=") { 3 }
        else if lex.consume_byte(b'<') { 4 }
        else if lex.consume_byte(b'>') { 5 }
        else { return Err(()); };

    let rhs = parse_expr(lex, vars)?;
    Ok(match cmp {
        0 => lhs == rhs, 1 => lhs != rhs,
        2 => lhs <= rhs, 3 => lhs >= rhs,
        4 => lhs < rhs,  _ => lhs > rhs,
    })
}

/// Разбор строкового литерала "..." (без escape-последовательностей)
fn parse_string_literal<'a>(lex: &mut Lexer<'a>) -> Result<&'a [u8], ()> {
    lex.skip_ws();
    if !lex.consume_byte(b'"') { return Err(()); }
    let start = lex.pos;
    while matches!(lex.peek(), Some(b) if b != b'"') {
        lex.pos += 1;
    }
    let text = &lex.bytes[start..lex.pos];
    if !lex.consume_byte(b'"') { return Err(()); }
    Ok(text)
}

/// Вывод программы RealX - захваченные строки print(), с ограничением по
/// кол-ву (см. заголовок файла - "ультра-базовый" интерпретатор без скролла)
pub struct RealXOutput {
    lines: [[u8; LINE_LEN]; MAX_LINES],
    len: [usize; MAX_LINES],
    count: usize,
    pub error: bool,
}

impl RealXOutput {
    fn new() -> Self {
        RealXOutput { lines: [[0; LINE_LEN]; MAX_LINES], len: [0; MAX_LINES], count: 0, error: false }
    }

    fn push(&mut self, text: &[u8]) {
        if self.count >= MAX_LINES {
            return; // Лишний вывод отбрасывается - см. заголовок файла
        }
        let n = text.len().min(LINE_LEN);
        self.lines[self.count][..n].copy_from_slice(&text[..n]);
        self.len[self.count] = n;
        self.count += 1;
    }

    fn push_error(&mut self, line: usize) {
        let mut buf = [0u8; 10];
        let num = crate::utils::u32_to_dec_str((line + 1) as u32, &mut buf);
        let mut msg: [u8; LINE_LEN] = [0; LINE_LEN];
        let prefix = b"Error on line ";
        let mut pos = 0;
        for &b in prefix { if pos < LINE_LEN { msg[pos] = b; pos += 1; } }
        for &b in num.as_bytes() { if pos < LINE_LEN { msg[pos] = b; pos += 1; } }
        self.push(&msg[..pos]);
        self.error = true;
    }

    pub fn count(&self) -> usize { self.count }

    pub fn line_str(&self, i: usize) -> &str {
        core::str::from_utf8(&self.lines[i][..self.len[i]]).unwrap_or("")
    }
}

/// true, если строка начинается с ключевого слова if/while как отдельным
/// токеном (а не просто текстовым префиксом - напр. "ifoo = 1" не в счёт)
fn opens_block(trimmed: &str) -> bool {
    for kw in ["if", "while"] {
        let bytes = trimmed.as_bytes();
        if bytes.len() > kw.len() && &bytes[..kw.len()] == kw.as_bytes() {
            let next = bytes[kw.len()];
            if !next.is_ascii_alphanumeric() && next != b'_' {
                return true;
            }
        }
    }
    false
}

/// Ищет строку с соответствующим `end` для блока if/while, открытого на
/// строке `open_line` (сканирует вперёд, считая уровень вложенности)
fn skip_to_matching_end(source: &Editor, open_line: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut i = open_line + 1;
    while i < MAX_LINES {
        let trimmed = source.line_str(i).trim();
        if opens_block(trimmed) {
            depth += 1;
        } else if trimmed == "end" {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Выполняет программу RealX, записанную в редакторе `source`, и возвращает
/// захваченный вывод (или сообщение об ошибке с error=true)
pub fn run(source: &Editor) -> RealXOutput {
    let mut out = RealXOutput::new();
    let mut vars = VarTable::new();
    let mut block_stack: [(usize, bool); MAX_BLOCK_DEPTH] = [(0, false); MAX_BLOCK_DEPTH];
    let mut depth = 0usize;
    let mut pc = 0usize;
    let mut steps = 0usize;

    while pc < MAX_LINES {
        steps += 1;
        if steps > MAX_STEPS {
            out.push(b"Error: step limit hit (infinite loop?)");
            out.error = true;
            return out;
        }

        let line = source.line_str(pc);
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            pc += 1;
            continue;
        }

        if trimmed == "end" {
            if depth == 0 {
                out.push_error(pc);
                return out;
            }
            let (start, is_while) = block_stack[depth - 1];
            depth -= 1;
            pc = if is_while { start } else { pc + 1 };
            continue;
        }

        let mut lex = Lexer::new(trimmed.as_bytes());
        let ident = lex.read_ident();

        if ident == b"print" {
            if !lex.consume_byte(b'(') {
                out.push_error(pc);
                return out;
            }
            lex.skip_ws();
            let printed: Result<(), ()> = if lex.peek() == Some(b'"') {
                match parse_string_literal(&mut lex) {
                    Ok(text) => { out.push(text); Ok(()) }
                    Err(()) => Err(()),
                }
            } else {
                match parse_expr(&mut lex, &vars) {
                    Ok(value) => {
                        let mut buf = [0u8; 12];
                        let text = format_i64(value, &mut buf);
                        out.push(text.as_bytes());
                        Ok(())
                    }
                    Err(()) => Err(()),
                }
            };
            if printed.is_err() || !lex.consume_byte(b')') {
                out.push_error(pc);
                return out;
            }
            pc += 1;
        } else if ident == b"if" || ident == b"while" {
            let is_while = ident == b"while";
            let cond = match parse_condition(&mut lex, &vars) {
                Ok(c) => c,
                Err(()) => { out.push_error(pc); return out; }
            };
            if !lex.consume_byte(b':') {
                out.push_error(pc);
                return out;
            }

            if cond {
                if depth >= MAX_BLOCK_DEPTH {
                    out.push(b"Error: blocks nested too deep");
                    out.error = true;
                    return out;
                }
                block_stack[depth] = (pc, is_while);
                depth += 1;
                pc += 1;
            } else {
                match skip_to_matching_end(source, pc) {
                    Some(end_line) => pc = end_line + 1,
                    None => {
                        out.push(b"Error: missing 'end'");
                        out.error = true;
                        return out;
                    }
                }
            }
        } else if ident.is_empty() {
            out.push_error(pc);
            return out;
        } else {
            // Присваивание: IDENT = expr
            if !lex.consume_byte(b'=') {
                out.push_error(pc);
                return out;
            }
            match parse_expr(&mut lex, &vars) {
                Ok(value) => {
                    if vars.set(ident, value).is_err() {
                        out.push(b"Error: too many variables");
                        out.error = true;
                        return out;
                    }
                }
                Err(()) => { out.push_error(pc); return out; }
            }
            pc += 1;
        }
    }

    out
}

/// Форматирование числа в буфер (для print(expr) с числовым результатом)
fn format_i64<'a>(value: i64, buf: &'a mut [u8; 12]) -> &'a str {
    let (negative, magnitude) = if value < 0 { (true, (-value) as u32) } else { (false, value as u32) };
    let mut num_buf: [u8; 10] = [0u8; 10];
    let num_str = crate::utils::u32_to_dec_str(magnitude, &mut num_buf);

    let mut pos = 0;
    if negative {
        buf[0] = b'-';
        pos = 1;
    }
    for &b in num_str.as_bytes() {
        if pos >= buf.len() { break; }
        buf[pos] = b;
        pos += 1;
    }

    core::str::from_utf8(&buf[..pos]).unwrap_or("?")
}
