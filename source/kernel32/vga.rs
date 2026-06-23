const VGA_BUFFER: *mut u8 = 0xB8000 as *mut u8;
pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

static mut CURSOR_ROW: usize = 0;
static mut CURSOR_COL: usize = 0;

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
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

pub fn clear_screen() {
    for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
        unsafe {
            VGA_BUFFER.add(i * 2).write_volatile(b' ');
            VGA_BUFFER.add(i * 2 + 1).write_volatile(0x0F);
        }
    }
    unsafe {
        CURSOR_ROW = 0;
        CURSOR_COL = 0;
    }
}

fn scroll_up() {
    unsafe {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let src = (row * VGA_WIDTH + col) * 2;
                let dst = ((row - 1) * VGA_WIDTH + col) * 2;
                let ch = VGA_BUFFER.add(src).read_volatile();
                let cl = VGA_BUFFER.add(src + 1).read_volatile();
                VGA_BUFFER.add(dst).write_volatile(ch);
                VGA_BUFFER.add(dst + 1).write_volatile(cl);
            }
        }
        let last_row = (VGA_HEIGHT - 1) * VGA_WIDTH * 2;
        for col in 0..VGA_WIDTH {
            VGA_BUFFER.add(last_row + col * 2).write_volatile(b' ');
            VGA_BUFFER.add(last_row + col * 2 + 1).write_volatile(0x0F);
        }
    }
}

fn update_cursor() {
    unsafe {
        let pos = (CURSOR_ROW * VGA_WIDTH + CURSOR_COL) as u16;
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Fu8);
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") (pos & 0xFF) as u8);
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Eu8);
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") ((pos >> 8) & 0xFF) as u8);
    }
}

pub fn put_char(c: u8, color: Color) {
    let color_byte = color as u8;
    unsafe {
        if c == b'\n' {
            CURSOR_COL = 0;
            CURSOR_ROW += 1;
        } else if c == b'\r' {
            CURSOR_COL = 0;
        } else {
            let offset = (CURSOR_ROW * VGA_WIDTH + CURSOR_COL) * 2;
            VGA_BUFFER.add(offset).write_volatile(c);
            VGA_BUFFER.add(offset + 1).write_volatile(color_byte);
            CURSOR_COL += 1;
        }

        if CURSOR_COL >= VGA_WIDTH {
            CURSOR_COL = 0;
            CURSOR_ROW += 1;
        }

        while CURSOR_ROW >= VGA_HEIGHT {
            scroll_up();
            CURSOR_ROW = VGA_HEIGHT - 1;
        }

        update_cursor();
    }
}

pub fn print_str(s: &str, color: Color) {
    for byte in s.bytes() {
        put_char(byte, color);
    }
}

pub fn backspace() {
    unsafe {
        if CURSOR_COL > 0 {
            CURSOR_COL -= 1;
        } else if CURSOR_ROW > 0 {
            CURSOR_ROW -= 1;
            CURSOR_COL = VGA_WIDTH - 1;
        }
        let offset = (CURSOR_ROW * VGA_WIDTH + CURSOR_COL) * 2;
        VGA_BUFFER.add(offset).write_volatile(b' ');
        VGA_BUFFER.add(offset + 1).write_volatile(0x0F);
        update_cursor();
    }
}

pub fn write_char_at(row: usize, col: usize, byte: u8, color: Color) {
    if row >= VGA_HEIGHT || col >= VGA_WIDTH {
        return;
    }
    let offset = (row * VGA_WIDTH + col) * 2;
    unsafe {
        let vga_ptr = 0xB8000 as *mut u8;
        vga_ptr.add(offset).write_volatile(byte);
        vga_ptr.add(offset + 1).write_volatile(color as u8);
    }
}
