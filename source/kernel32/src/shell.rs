use crate::keyboard;
use crate::vga::{self, Color};

const HIST_MAX: usize = 10;
static mut HISTORY: [[u8; 64]; HIST_MAX] = [[0; 64]; HIST_MAX];
static mut HIST_COUNT: usize = 0;
static mut HIST_POS: isize = -1;

pub fn run() {
    vga::print_str("Type 'help' for commands.\n", vga::Color::LightGray);
    loop {
        vga::print_str("Realix> ", vga::Color::Green);
        
        let input = read_line_with_history();
        vga::put_char(b'\n', vga::Color::LightGray);
        
        // Сохраняем в историю
        let len = input.iter().position(|&c| c == 0).unwrap_or(64);
        if len > 0 {
            unsafe {
                if HIST_COUNT < HIST_MAX {
                    HISTORY[HIST_COUNT] = input;
                    HIST_COUNT += 1;
                } else {
                    for i in 0..HIST_MAX-1 { HISTORY[i] = HISTORY[i+1]; }
                    HISTORY[HIST_MAX-1] = input;
                }
                HIST_POS = HIST_COUNT as isize;
            }
        }
        
        let cmd = core::str::from_utf8(&input[..len]).unwrap_or("");
        execute(cmd);
    }
}

fn read_line_with_history() -> [u8; 64] {
    let mut buf = [0u8; 64];
    let mut pos = 0;

    loop {
        let key = keyboard::read_key_blocking();
        match key {
            b'\n' => {
                buf[pos] = 0;
                return buf;
            }
            b'\x08' => {
                if pos > 0 { pos -= 1; vga::backspace(); }
            }
            0x80 => { // Стрелка вверх
                unsafe {
                    if HIST_COUNT > 0 && HIST_POS > 0 {
                        HIST_POS -= 1;
                        while pos > 0 { pos -= 1; vga::backspace(); }
                        let h = &HISTORY[HIST_POS as usize];
                        pos = copy_from_history(&mut buf, h);
                    }
                }
            }
            0x81 => { // Стрелка вниз
                unsafe {
                    if (HIST_POS as usize) < HIST_COUNT.saturating_sub(1) {
                        HIST_POS += 1;
                        while pos > 0 { pos -= 1; vga::backspace(); }
                        let h = &HISTORY[HIST_POS as usize];
                        pos = copy_from_history(&mut buf, h);
                    } else if HIST_COUNT > 0 {
                        HIST_POS = HIST_COUNT as isize;
                        while pos > 0 { pos -= 1; vga::backspace(); }
                        pos = 0;
                    }
                }
            }
            c if pos < 63 && c >= 0x20 => {
                buf[pos] = c;
                pos += 1;
                vga::put_char(c, vga::Color::LightGray);
            }
            _ => {}
        }
    }
}

fn copy_from_history(buf: &mut [u8; 64], hist: &[u8; 64]) -> usize {
    let mut i = 0;
    while i < 64 && hist[i] != 0 {
        buf[i] = hist[i];
        vga::put_char(hist[i], vga::Color::LightGray);
        i += 1;
    }
    i
}

fn execute(input: &str) {
    match input {
        "help" => {
            vga::print_str("\nCommands:\n\n", vga::Color::Cyan);
            let cmds = [
                ("  clear", "Clear screen"),
                ("  echo", "Print text"),
                ("  meminfo", "Memory info"),
                ("  calc", "Calculator"),
                ("  snake", "Snake game"),
                ("  matrix", "Matrix rain"),
                ("  fetch", "System info"),
                ("  netinfo", "Network MAC"),
                ("  arp", "Send ARP request"),
                ("  history", "Command history"),
                ("  help", "This help"),
                ("  time", "Show date and time"),
            ];
            for (cmd, desc) in cmds.iter() {
                vga::print_str(cmd, vga::Color::Green);
                vga::print_str(" - ", vga::Color::DarkGray);
                vga::print_str(desc, vga::Color::LightGray);
                vga::put_char(b'\n', vga::Color::LightGray);
            }
            vga::put_char(b'\n', vga::Color::LightGray);
        }
        "time" => {
            crate::rtc::print_time();
            crate::rtc::print_date();
        }
        "clear" => vga::clear_screen(),
        "history" => {
            unsafe {
                for i in 0..HIST_COUNT {
                    let num = i + 1;
                    if num < 10 { vga::put_char(b' ', vga::Color::DarkGray); }
                    vga::put_char(b'0' + (num/10) as u8, vga::Color::DarkGray);
                    vga::put_char(b'0' + (num%10) as u8, vga::Color::DarkGray);
                    vga::print_str(" ", vga::Color::White);
                    for j in 0..64 {
                        if HISTORY[i][j] == 0 { break; }
                        vga::put_char(HISTORY[i][j], vga::Color::LightGray);
                    }
                    vga::put_char(b'\n', vga::Color::LightGray);
                }
            }
        }
        "snake" => { crate::snake::run(); }
        "matrix" => { crate::matrix::run(); }
        "fetch" => { crate::fetch::run(); }
        "arp" => { crate::arp::send_arp_request([10,0,2,2]); }
        _ if input.starts_with("echo ") => {
            vga::print_str(&input[5..], vga::Color::LightGray);
            vga::put_char(b'\n', vga::Color::LightGray);
        }
        "meminfo" => vga::print_str("Memory: 640 KB\n", vga::Color::Yellow),
        _ if input.starts_with("calc ") => { crate::calc::run(&input[5..]); }
        "netinfo" => {
            if crate::rtl8139::is_ready() {
                let mut mac = [0u8;6];
                crate::rtl8139::get_mac(&mut mac);
                vga::print_str("MAC: ", vga::Color::Green);
                for i in 0..6 {
                    let h = b"0123456789ABCDEF";
                    vga::put_char(h[(mac[i]>>4) as usize], vga::Color::White);
                    vga::put_char(h[(mac[i]&0xF) as usize], vga::Color::White);
                    if i<5 { vga::put_char(b':', vga::Color::White); }
                }
                vga::put_char(b'\n', vga::Color::LightGray);
            } else { vga::print_str("No network card\n", vga::Color::Red); }
        }
        "" => {}
        _ => vga::print_str("Unknown command\n", vga::Color::Red),
    }
}
