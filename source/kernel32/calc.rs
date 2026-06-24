use crate::vga;

pub fn run(input: &str) {
    // input = "12 + 5" (без "calc ")
    let parts: &[u8] = input.as_bytes();
    
    // Ищем оператор (+, -, *, /)
    let mut op_pos = 0;
    let mut op = b' ';
    
    for (i, &c) in parts.iter().enumerate() {
        if c == b'+' || c == b'-' || c == b'*' || c == b'/' {
            op_pos = i;
            op = c;
            break;
        }
    }
    
    if op == b' ' {
        vga::print_str("Usage: calc <num1> <+|-|*|/> <num2>\n", vga::Color::Red);
        return;
    }
    
    // Парсим первое число
    let num1_str = &input[..op_pos].trim();
    let num1 = parse_int(num1_str);
    
    // Парсим второе число
    let num2_str = &input[op_pos + 1..].trim();
    let num2 = parse_int(num2_str);
    
    match (num1, num2) {
        (Some(a), Some(b)) => {
            let result = match op {
                b'+' => a.wrapping_add(b),
                b'-' => a.wrapping_sub(b),
                b'*' => a.wrapping_mul(b),
                b'/' => {
                    if b == 0 {
                        vga::print_str("Error: Division by zero\n", vga::Color::Red);
                        return;
                    }
                    a.wrapping_div(b)
                }
                _ => {
                    vga::print_str("Unknown operator\n", vga::Color::Red);
                    return;
                }
            };
            
            // Выводим результат
            vga::print_str("Result: ", vga::Color::Green);
            print_int(result);
            vga::put_char(b'\n', vga::Color::LightGray);
        }
        _ => {
            vga::print_str("Error: Invalid numbers\n", vga::Color::Red);
        }
    }
}

fn parse_int(s: &str) -> Option<i32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    
    let mut result: i32 = 0;
    let mut negative = false;
    let bytes = s.as_bytes();
    let mut start = 0;
    
    if bytes[0] == b'-' {
        negative = true;
        start = 1;
    }
    
    for &c in &bytes[start..] {
        if c >= b'0' && c <= b'9' {
            result = result.wrapping_mul(10).wrapping_add((c - b'0') as i32);
        } else {
            return None;
        }
    }
    
    if negative {
        Some(-result)
    } else {
        Some(result)
    }
}

fn print_int(mut n: i32) {
    if n == 0 {
        vga::put_char(b'0', vga::Color::White);
        return;
    }
    
    if n < 0 {
        vga::put_char(b'-', vga::Color::White);
        n = -n;
    }
    
    let mut buf: [u8; 11] = [0; 11]; // максимум 10 цифр + знак
    let mut pos = 10;
    
    while n > 0 && pos > 0 {
        pos -= 1;
        buf[pos] = (n % 10) as u8 + b'0';
        n /= 10;
    }
    
    for &c in &buf[pos..] {
        if c != 0 {
            vga::put_char(c, vga::Color::White);
        }
    }
}
