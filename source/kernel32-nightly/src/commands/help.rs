// © Realix > Command: Help
// (22.07.26) v0.1
// ================

// Подключение функций
use crate::drivers::vga::{self, Color};

/// Выводит список доступных команд, сгруппированный по категориям
pub fn show() {
    vga::print_line("Commands:\n", Color::LightCyan);

    vga::print_line("  [Base]\n", Color::Cyan);
    vga::print_line("> help      - Show this manual\n", Color::LightGray);
    vga::print_line("> clear/cls - Clear screen\n", Color::LightGray);
    vga::print_line("> echo [t]  - Print text to console\n", Color::LightGray);
    vga::print_line("> about     - Show info about Realix\n", Color::LightGray);
    vga::print_line("> beep      - Beep via PC speaker\n", Color::LightGray);
    vga::print_line("> uptime    - Show uptime (seconds)\n", Color::LightGray);
    vga::print_line("> meminfo   - Show memory information\n", Color::LightGray);
    vga::print_line("> sysinfo   - System information\n", Color::LightGray);

    vga::print_line("  [Text]\n", Color::Cyan);
    vga::print_line("> len <t>        - Length of <t>\n", Color::LightGray);
    vga::print_line("> upper <t>      - <t> to upper case\n", Color::LightGray);
    vga::print_line("> lower <t>      - <t> to lower case\n", Color::LightGray);
    vga::print_line("> reverse <t>    - Reverse <t>\n", Color::LightGray);
    vga::print_line("> repeat <n> <t> - Repeat <t> <n> times\n", Color::LightGray);
    vga::print_line("> ascii <0-255>  - Char by ASCII code\n", Color::LightGray);

    vga::print_line("  [Numbers]\n", Color::Cyan);
    vga::print_line("> calc <a> <+ - * /> <b> - Calculator\n", Color::LightGray);
    vga::print_line("> hex <num>  - <num> to hexadecimal\n", Color::LightGray);
    vga::print_line("> fib <0-46> - Nth Fibonacci number\n", Color::LightGray);

    vga::print_line("  [System]\n", Color::Cyan);
    vga::print_line("> regs  - Show CPU registers snapshot\n", Color::LightGray);
    vga::print_line("> time  - Show RTC time\n", Color::LightGray);
    vga::print_line("> date  - Show RTC date\n", Color::LightGray);
    vga::print_line("> vga   - Colored rectangles demo\n", Color::LightGray);
    vga::print_line("> panic - Show panic screen and halt\n", Color::LightGray);

    vga::print_line("  [Fat12]\n", Color::Cyan);
    vga::print_line("> ls          - List root directory\n", Color::LightGray);
    vga::print_line("> load <f>    - Load file into a RAM buffer\n", Color::LightGray);
    vga::print_line("> type <f>    - Print file as text\n", Color::LightGray);
    vga::print_line("> hexdump <f> - Hex dump of file\n", Color::LightGray);
    vga::print_line("> exec <f>    - Run a 32-bit .rlx app\n", Color::LightGray);

    vga::print_line("  [Fun]\n", Color::Cyan);
    vga::print_line("> matrix    - Show matrix rain\n", Color::LightGray);
    vga::print_line("> nova [-a] - Chat with the local NovaAI model\n", Color::LightGray);

    vga::print_line("  [Power]\n", Color::Cyan);
    vga::print_line("> reboot    - Reboot PC\n", Color::LightGray);
    vga::print_line("> shutdown  - Power off PC\n", Color::LightGray);
}