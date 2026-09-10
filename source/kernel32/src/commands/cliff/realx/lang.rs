// © Realix > Cliff: RealX - ультра-базовый язык программирования
// ================
// ❗️ Синтаксис нарочно похож на Python (переменные, print(...), if/while с
// операторами сравнения, строки в "" или ''), но с явным `end` вместо
// отступов - настоящий Python-style off-side rule (отступы как границы
// блоков) требует полноценного токенайзера с отслеживанием уровня
// вложенности; `end` даёт тот же читаемый результат при заметно проще
// построчном интерпретаторе - см. run() ниже
// ❗️ Ограничения (осознанно, "ультра-базовый" язык):
//    - Два типа значений: целые (i64) и строки (до STR_CAP байт, отдельно
//      от переменных - не Vec/String, фиксированный буфер в каждом Value)
//    - "+" работает для пары int (сложение) ИЛИ пары str (конкатенация) -
//      смешивать типы в одном выражении - ошибка; "- * /" только для int
//    - Сравнение (== != < <= > >=) - только между двумя int или двумя str
//      (лексикографически); смешивать типы - ошибка
//    - До 16 переменных, имя до 8 символов
//    - if/while - ровно одно условие сравнения, без elif/else, без вложенных
//      выражений сложнее вида (a + b * c) со скобками
//    - Нет пользовательских функций (def) - только print/input, условия,
//      циклы и присваивания
//    - input() блокирует выполнение и рисует поле ввода прямо в открытом
//      окне вывода (см. cliff::realx_read_input) - реентерабельности нет,
//      это обычный синхронный вызов клавиатуры, как и везде в kernel32
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

// Вместимость строкового значения и ввода input() - размер строки вывода
pub const STR_CAP: usize = LINE_LEN;
pub const INPUT_CAP: usize = LINE_LEN;

/// Значение RealX - целое число или строка (см. заголовок файла)
#[derive(Clone, Copy)]
enum Value {
    Int(i64),
    Str([u8; STR_CAP], usize),
}

impl Value {
    fn from_bytes(bytes: &[u8]) -> Value {
        let mut buf = [0u8; STR_CAP];
        let n = bytes.len().min(STR_CAP);
        buf[..n].copy_from_slice(&bytes[..n]);
        Value::Str(buf, n)
    }
}

fn as_int(v: Value) -> Result<i64, ()> {
    match v {
        Value::Int(n) => Ok(n),
        Value::Str(..) => Err(()),
    }
}

/// "+" - сложение для пары int, конкатенация для пары str; смешивать типы
/// (или переполнить строку сверх STR_CAP - см. заголовок файла) - ошибка
fn add_values(a: Value, b: Value) -> Result<Value, ()> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.checked_add(y).ok_or(())?)),
        (Value::Str(abuf, alen), Value::Str(bbuf, blen)) => {
            let mut buf = [0u8; STR_CAP];
            let mut pos = 0;
            for &byte in &abuf[..alen] {
                if pos >= STR_CAP { break; }
                buf[pos] = byte;
                pos += 1;
            }
            for &byte in &bbuf[..blen] {
                if pos >= STR_CAP { break; }
                buf[pos] = byte;
                pos += 1;
            }
            Ok(Value::Str(buf, pos))
        }
        _ => Err(()),
    }
}

/// Сравнение - между двумя int (числовое) или двумя str (побайтовое,
/// лексикографическое); смешивать типы - ошибка
fn compare(cmp: u8, lhs: Value, rhs: Value) -> Result<bool, ()> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(match cmp {
            0 => a == b, 1 => a != b, 2 => a <= b, 3 => a >= b, 4 => a < b, _ => a > b,
        }),
        (Value::Str(abuf, alen), Value::Str(bbuf, blen)) => {
            let a = &abuf[..alen];
            let b = &bbuf[..blen];
            Ok(match cmp {
                0 => a == b, 1 => a != b, 2 => a <= b, 3 => a >= b, 4 => a < b, _ => a > b,
            })
        }
        _ => Err(()),
    }
}

/// Составное присваивание (x += / -= / *= / /= expr) - та же арифметика,
/// что и обычные операторы (add_values для "+", as_int для остальных, т.к.
/// -,*,/ определены только для int - см. заголовок файла)
fn apply_compound(op: u8, current: Value, rhs: Value) -> Result<Value, ()> {
    match op {
        b'+' => add_values(current, rhs),
        b'-' => Ok(Value::Int(as_int(current)?.checked_sub(as_int(rhs)?).ok_or(())?)),
        b'*' => Ok(Value::Int(as_int(current)?.checked_mul(as_int(rhs)?).ok_or(())?)),
        b'/' => {
            let divisor = as_int(rhs)?;
            if divisor == 0 { return Err(()); }
            Ok(Value::Int(as_int(current)?.checked_div(divisor).ok_or(())?))
        }
        _ => Err(()),
    }
}

#[derive(Clone, Copy)]
struct Var {
    name: [u8; VAR_NAME_LEN],
    name_len: u8,
    value: Value,
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

    fn get(&self, name: &[u8]) -> Option<Value> {
        self.find(name).map(|i| self.vars[i].unwrap().value)
    }

    /// Ok(()) при успехе; Err(()) если имя слишком длинное или таблица полна
    fn set(&mut self, name: &[u8], value: Value) -> Result<(), ()> {
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

/// Разбор строкового литерала "..." или '...' (без escape-последовательностей;
/// закрывающая кавычка должна совпадать с открывающей)
fn parse_string_literal<'a>(lex: &mut Lexer<'a>) -> Result<&'a [u8], ()> {
    lex.skip_ws();
    let quote = match lex.peek() {
        Some(b @ b'"') | Some(b @ b'\'') => b,
        _ => return Err(()),
    };
    lex.pos += 1;
    let start = lex.pos;
    while matches!(lex.peek(), Some(b) if b != quote) {
        lex.pos += 1;
    }
    let text = &lex.bytes[start..lex.pos];
    if !lex.consume_byte(quote) {
        return Err(());
    }
    Ok(text)
}

/// Контекст вычисления выражения - доступ к переменным (только чтение - см.
/// run()/VarTable::set, изменение происходит отдельно после вычисления),
/// вывод программы (input() пишет в него подсказку/введённый текст) и
/// функция чтения ввода (см. cliff::realx_read_input - рисует поле ввода и
/// блокирующе читает клавиатуру)
struct EvalCtx<'a> {
    vars: &'a VarTable,
    out: &'a mut RealXOutput,
    read_input: &'a mut dyn FnMut(&mut RealXOutput) -> ([u8; INPUT_CAP], usize),
}

fn parse_expr(lex: &mut Lexer, ctx: &mut EvalCtx) -> Result<Value, ()> {
    let mut value = parse_term(lex, ctx)?;
    loop {
        lex.skip_ws();
        match lex.peek() {
            Some(b'+') => { lex.pos += 1; value = add_values(value, parse_term(lex, ctx)?)?; }
            Some(b'-') => {
                lex.pos += 1;
                let rhs = as_int(parse_term(lex, ctx)?)?;
                value = Value::Int(as_int(value)?.checked_sub(rhs).ok_or(())?);
            }
            _ => break,
        }
    }
    Ok(value)
}

fn parse_term(lex: &mut Lexer, ctx: &mut EvalCtx) -> Result<Value, ()> {
    let mut value = parse_factor(lex, ctx)?;
    loop {
        lex.skip_ws();
        match lex.peek() {
            Some(b'*') => {
                lex.pos += 1;
                let rhs = as_int(parse_factor(lex, ctx)?)?;
                value = Value::Int(as_int(value)?.checked_mul(rhs).ok_or(())?);
            }
            Some(b'/') => {
                lex.pos += 1;
                let rhs = as_int(parse_factor(lex, ctx)?)?;
                if rhs == 0 { return Err(()); }
                value = Value::Int(as_int(value)? / rhs);
            }
            _ => break,
        }
    }
    Ok(value)
}

fn parse_factor(lex: &mut Lexer, ctx: &mut EvalCtx) -> Result<Value, ()> {
    lex.skip_ws();
    match lex.peek() {
        Some(b'-') => { lex.pos += 1; Ok(Value::Int(-as_int(parse_factor(lex, ctx)?)?)) }
        Some(b'(') => {
            lex.pos += 1;
            let v = parse_expr(lex, ctx)?;
            if !lex.consume_byte(b')') { return Err(()); }
            Ok(v)
        }
        Some(b'"') | Some(b'\'') => {
            let text = parse_string_literal(lex)?;
            Ok(Value::from_bytes(text))
        }
        Some(b) if b.is_ascii_digit() => {
            let start = lex.pos;
            while matches!(lex.peek(), Some(b) if b.is_ascii_digit()) { lex.pos += 1; }
            Ok(Value::Int(parse_i64(&lex.bytes[start..lex.pos])?))
        }
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
            let name = lex.read_ident();
            if name == b"input" {
                parse_input_call(lex, ctx)
            } else {
                ctx.vars.get(name).ok_or(())
            }
        }
        _ => Err(()),
    }
}

/// input() / input("подсказка") - подсказка (если есть) выводится как
/// обычная строка вывода, затем (ctx.read_input) рисует поле и блокирующе
/// читает клавиатуру; введённый текст также попадает в вывод (эхо) и
/// возвращается как значение-строка
fn parse_input_call(lex: &mut Lexer, ctx: &mut EvalCtx) -> Result<Value, ()> {
    if !lex.consume_byte(b'(') { return Err(()); }

    lex.skip_ws();
    if matches!(lex.peek(), Some(b'"') | Some(b'\'')) {
        let prompt = parse_string_literal(lex)?;
        ctx.out.push(prompt);
    }
    if !lex.consume_byte(b')') { return Err(()); }

    let (buf, len) = (ctx.read_input)(ctx.out);
    ctx.out.push(&buf[..len]);
    Ok(Value::from_bytes(&buf[..len]))
}

/// Разбор условия сравнения `<expr> <op> <expr>` - ровно один оператор
fn parse_condition(lex: &mut Lexer, ctx: &mut EvalCtx) -> Result<bool, ()> {
    let lhs = parse_expr(lex, ctx)?;
    lex.skip_ws();

    let cmp = if lex.consume_str(b"==") { 0 }
        else if lex.consume_str(b"!=") { 1 }
        else if lex.consume_str(b"<=") { 2 }
        else if lex.consume_str(b">=") { 3 }
        else if lex.consume_byte(b'<') { 4 }
        else if lex.consume_byte(b'>') { 5 }
        else { return Err(()); };

    let rhs = parse_expr(lex, ctx)?;
    compare(cmp, lhs, rhs)
}

/// Вывод программы RealX - захваченные строки print()/input(), с
/// ограничением по кол-ву (см. заголовок файла - "ультра-базовый"
/// интерпретатор без скролла своего вывода)
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
/// захваченный вывод (или сообщение об ошибке с error=true). `read_input` -
/// см. EvalCtx/cliff::realx_read_input - вызывается при каждом input()
pub fn run(
    source: &Editor,
    mut read_input: impl FnMut(&mut RealXOutput) -> ([u8; INPUT_CAP], usize),
) -> RealXOutput {
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
            let value = {
                let mut ctx = EvalCtx { vars: &vars, out: &mut out, read_input: &mut read_input };
                parse_expr(&mut lex, &mut ctx)
            };
            match value {
                Ok(Value::Int(n)) => {
                    let mut buf = [0u8; 12];
                    let text = format_i64(n, &mut buf);
                    out.push(text.as_bytes());
                }
                Ok(Value::Str(buf, len)) => out.push(&buf[..len]),
                Err(()) => { out.push_error(pc); return out; }
            }
            if !lex.consume_byte(b')') {
                out.push_error(pc);
                return out;
            }
            pc += 1;
        } else if ident == b"if" || ident == b"while" {
            let is_while = ident == b"while";
            let cond = {
                let mut ctx = EvalCtx { vars: &vars, out: &mut out, read_input: &mut read_input };
                parse_condition(&mut lex, &mut ctx)
            };
            let cond = match cond {
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
            // Присваивание: IDENT = expr, либо составное IDENT += / -= / *= / /= expr
            let compound_op: Option<u8> = if lex.consume_str(b"+=") { Some(b'+') }
                else if lex.consume_str(b"-=") { Some(b'-') }
                else if lex.consume_str(b"*=") { Some(b'*') }
                else if lex.consume_str(b"/=") { Some(b'/') }
                else { None };

            if compound_op.is_none() && !lex.consume_byte(b'=') {
                out.push_error(pc);
                return out;
            }

            let rhs = {
                let mut ctx = EvalCtx { vars: &vars, out: &mut out, read_input: &mut read_input };
                parse_expr(&mut lex, &mut ctx)
            };

            let result = rhs.and_then(|rhs_val| match compound_op {
                None => Ok(rhs_val),
                Some(op) => {
                    let current = vars.get(ident).ok_or(())?;
                    apply_compound(op, current, rhs_val)
                }
            });

            match result {
                Ok(v) => {
                    if vars.set(ident, v).is_err() {
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
