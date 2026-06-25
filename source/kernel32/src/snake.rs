use crate::vga;

const WIDTH: usize = 40;
const HEIGHT: usize = 20;
const MAX_LEN: usize = 100;

static mut SNAKE_X: [usize; MAX_LEN] = [0; MAX_LEN];
static mut SNAKE_Y: [usize; MAX_LEN] = [0; MAX_LEN];
static mut LEN: usize = 3;
static mut FOOD_X: usize = 10;
static mut FOOD_Y: usize = 10;
static mut SCORE: usize = 0;
static mut GAME_OVER: bool = false;

pub fn run() {
    vga::clear_screen();
    
    unsafe {
        SNAKE_X[0] = 5; SNAKE_Y[0] = 10;
        SNAKE_X[1] = 4; SNAKE_Y[1] = 10;
        SNAKE_X[2] = 3; SNAKE_Y[2] = 10;
        LEN = 3;
        SCORE = 0;
        GAME_OVER = false;
    }
    
    draw_border();
    draw_all();
    
    loop {
        if unsafe { GAME_OVER } {
            draw_game_over();
            // Ждём R или Q
            loop {
                if let Some(key) = wait_key() {
                    if key == b'r' {
                        vga::clear_screen();
                        draw_border();
                        unsafe {
                            SNAKE_X[0] = 5; SNAKE_Y[0] = 10;
                            SNAKE_X[1] = 4; SNAKE_Y[1] = 10;
                            SNAKE_X[2] = 3; SNAKE_Y[2] = 10;
                            LEN = 3;
                            SCORE = 0;
                            GAME_OVER = false;
                        }
                        draw_all();
                        break;
                    }
                    if key == b'q' {
                        vga::clear_screen();
                        return;
                    }
                }
            }
            continue;
        }
        
        // Ждём клавишу направления
        if let Some(key) = wait_key() {
            match key {
                b'w' => { tick(0, -1); }
                b's' => { tick(0, 1); }
                b'a' => { tick(-1, 0); }
                b'd' => { tick(1, 0); }
                b'q' => { vga::clear_screen(); return; }
                _ => {}
            }
        }
    }
}

fn draw_border() {
    for x in 0..WIDTH + 2 {
        vga::write_char_at(0, x, 0xCD, vga::Color::Blue);
        vga::write_char_at(HEIGHT + 1, x, 0xCD, vga::Color::Blue);
    }
    for y in 0..HEIGHT + 2 {
        vga::write_char_at(y, 0, 0xBA, vga::Color::Blue);
        vga::write_char_at(y, WIDTH + 1, 0xBA, vga::Color::Blue);
    }
    vga::write_char_at(0, 0, 0xC9, vga::Color::Blue);
    vga::write_char_at(0, WIDTH + 1, 0xBB, vga::Color::Blue);
    vga::write_char_at(HEIGHT + 1, 0, 0xC8, vga::Color::Blue);
    vga::write_char_at(HEIGHT + 1, WIDTH + 1, 0xBC, vga::Color::Blue);
}

fn draw_all() {
    unsafe {
        // Очищаем поле
        for y in 1..=HEIGHT {
            for x in 1..=WIDTH {
                vga::write_char_at(y, x, b' ', vga::Color::Black);
            }
        }
        // Рисуем змейку
        for i in 0..LEN {
            let sx = SNAKE_X[i] + 1;
            let sy = SNAKE_Y[i] + 1;
            let ch = if i == 0 { 0xFE } else { b'o' };
            let color = if i == 0 { vga::Color::LightGreen } else { vga::Color::Green };
            vga::write_char_at(sy, sx, ch, color);
        }
        // Еда
        vga::write_char_at(FOOD_Y + 1, FOOD_X + 1, b'*', vga::Color::Red);
    }
    draw_score();
}

fn draw_score() {
    let msg = b"Score: ";
    for (i, &c) in msg.iter().enumerate() {
        vga::write_char_at(HEIGHT + 2, i + 1, c, vga::Color::Yellow);
    }
    unsafe {
        let mut s = SCORE;
        if s == 0 {
            vga::write_char_at(HEIGHT + 2, 8, b'0', vga::Color::Yellow);
        } else {
            let mut pos = 10;
            while s > 0 && pos >= 8 {
                vga::write_char_at(HEIGHT + 2, pos, (s % 10) as u8 + b'0', vga::Color::Yellow);
                s /= 10;
                if pos > 8 { pos -= 1; }
            }
        }
    }
}

fn draw_game_over() {
    let msg = "GAME OVER! R=Restart Q=Quit";
    let x = (WIDTH + 2 - msg.len()) / 2;
    let y = HEIGHT / 2 + 1;
    for (i, &c) in msg.as_bytes().iter().enumerate() {
        vga::write_char_at(y, x + i, c, vga::Color::Red);
    }
}

fn tick(dx: i32, dy: i32) {
    unsafe {
        // Нельзя развернуться в себя
        let old_dx = SNAKE_X[0] as i32 - SNAKE_X[1] as i32;
        let old_dy = SNAKE_Y[0] as i32 - SNAKE_Y[1] as i32;
        if dx == -old_dx && dy == -old_dy && LEN > 1 {
            return; // разворот запрещён
        }
        
        let old_tail_x = SNAKE_X[LEN - 1];
        let old_tail_y = SNAKE_Y[LEN - 1];
        
        for i in (1..LEN).rev() {
            SNAKE_X[i] = SNAKE_X[i - 1];
            SNAKE_Y[i] = SNAKE_Y[i - 1];
        }
        
        SNAKE_X[0] = (SNAKE_X[0] as i32 + dx) as usize;
        SNAKE_Y[0] = (SNAKE_Y[0] as i32 + dy) as usize;
        
        let hx = SNAKE_X[0];
        let hy = SNAKE_Y[0];
        
        if hx >= WIDTH || hy >= HEIGHT {
            GAME_OVER = true;
            return;
        }
        
        for i in 1..LEN {
            if SNAKE_X[i] == hx && SNAKE_Y[i] == hy {
                GAME_OVER = true;
                return;
            }
        }
        
        if hx == FOOD_X && hy == FOOD_Y {
            if LEN < MAX_LEN {
                SNAKE_X[LEN] = old_tail_x;
                SNAKE_Y[LEN] = old_tail_y;
                LEN += 1;
            }
            SCORE += 10;
            loop {
                FOOD_X = (hx * 13 + hy * 7 + 123) % WIDTH;
                FOOD_Y = (hx * 17 + hy * 11 + 456) % HEIGHT;
                let mut ok = true;
                for i in 0..LEN {
                    if SNAKE_X[i] == FOOD_X && SNAKE_Y[i] == FOOD_Y {
                        ok = false;
                        break;
                    }
                }
                if ok { break; }
            }
        }
        
        draw_all();
    }
}

fn wait_key() -> Option<u8> {
    // Блокирующее ожидание клавиши
    loop {
        unsafe {
            let s: u8;
            core::arch::asm!("in al, 0x64", out("al") s);
            if s & 1 != 0 && s & 0x20 == 0 {
                let sc: u8;
                core::arch::asm!("in al, 0x60", out("al") sc);
                if sc & 0x80 == 0 {
                    return match sc {
                        0x11 => Some(b'w'),
                        0x1F => Some(b's'),
                        0x1E => Some(b'a'),
                        0x20 => Some(b'd'),
                        0x10 => Some(b'q'),
                        0x13 => Some(b'r'),
                        _ => None,
                    };
                }
            }
        }
    }
}
