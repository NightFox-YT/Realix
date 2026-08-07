// © Realix > Desktop32 v2.0 — Window Manager, Taskbar, Desktop, Terminal
// (28.07.26) — Fixed keyboard, PS/2 Mouse Driver, VGA Direct Render
// ======================================================================

// ---------- Системные вызовы INT 0x80 ----------
#define SYS_PRINT_STRING  1
#define SYS_PUTCHAR       2
#define SYS_EXIT          3
#define SYS_READ_KEY      4   // блокирующий
#define SYS_CLEAR         5
#define SYS_GET_SYSINFO   6
#define SYS_MOUSE_INIT    8
#define SYS_READ_MOUSE    9
#define SYS_READ_KEY_NB  10   // неблокирующий (0 = нет клавиши)

// ---------- Специальные коды клавиш ----------
#define KEY_ESC        0x1B
#define KEY_BACKSPACE  0x08
#define KEY_ENTER      0x0D
#define KEY_TAB        0x09
#define KEY_UP         0x80
#define KEY_DOWN       0x81
#define KEY_F1         0xF1
#define KEY_F2         0xF2
#define KEY_F3         0xF3
#define KEY_F4         0xF4

// ---------- VGA цвета (fg) ----------
#define C_BLACK    0
#define C_BLUE     1
#define C_GREEN    2
#define C_CYAN     3
#define C_RED      4
#define C_MAGENTA  5
#define C_BROWN    6
#define C_LGRAY    7
#define C_DGRAY    8
#define C_LBLUE    9
#define C_LGREEN  10
#define C_LCYAN   11
#define C_LRED    12
#define C_PINK    13
#define C_YELLOW  14
#define C_WHITE   15

// ---------- VGA Text Screen ----------
#define VGA_W   80
#define VGA_H   25
static volatile unsigned short * const VGA = (volatile unsigned short *)0xB8000;

// ---------- Состояния рабочего стола ----------
#define S_DESKTOP   0
#define S_TERMINAL  1
#define S_SYSINFO   2
#define S_FILES     3

// ---------- Мышь ----------
static int mouse_x = 40, mouse_y = 12;
static int mouse_lbtn = 0, mouse_prev_lbtn = 0;

// ---------- Терминал ----------
static char  term_lines[10][80];
static int   term_line_count = 0;
static char  term_input[64];
static int   term_input_len = 0;

// ---------- Глобальное состояние ----------
static int   desktop_state = S_DESKTOP;
static unsigned int uptime = 0;

// ============================================================
// VGA ПРИМИТИВЫ (прямая запись в 0xB8000)
// ============================================================
static inline unsigned short vga_cell(char c, unsigned char fg, unsigned char bg) {
    return (unsigned short)(unsigned char)c | ((unsigned short)((bg << 4) | (fg & 0xF)) << 8);
}

static void vput(int x, int y, char c, unsigned char fg, unsigned char bg) {
    if (x >= 0 && x < VGA_W && y >= 0 && y < VGA_H)
        VGA[y * VGA_W + x] = vga_cell(c, fg, bg);
}

static void vfill(int x, int y, int w, int h, char c, unsigned char fg, unsigned char bg) {
    for (int row = y; row < y + h && row < VGA_H; row++)
        for (int col = x; col < x + w && col < VGA_W; col++)
            vput(col, row, c, fg, bg);
}

static void vprint(int x, int y, const char *s, unsigned char fg, unsigned char bg) {
    while (*s && x < VGA_W) vput(x++, y, *s++, fg, bg);
}

static void vcenter(int y, int w, const char *s, unsigned char fg, unsigned char bg) {
    int len = 0; while (s[len]) len++;
    int start = (w - len) / 2;
    vprint(start, y, s, fg, bg);
}

static int vstrlen(const char *s) { int n=0; while(s[n]) n++; return n; }

// ============================================================
// СИСТЕМНЫЕ ВЫЗОВЫ
// ============================================================
static inline unsigned int rlx_read_key(void) {
    unsigned int r;
    __asm__ __volatile__("int $0x80" : "=a"(r) : "a"(SYS_READ_KEY) : "memory");
    return r;
}

// Неблокирующий — возвращает 0 если нет нажатой клавиши
static inline unsigned int rlx_read_key_nb(void) {
    unsigned int r;
    __asm__ __volatile__("int $0x80" : "=a"(r) : "a"(SYS_READ_KEY_NB) : "memory");
    return r;
}

static inline void rlx_exit(void) {
    __asm__ __volatile__("int $0x80" :: "a"(SYS_EXIT) : "memory");
}

static inline void rlx_mouse_init(void) {
    __asm__ __volatile__("int $0x80" :: "a"(SYS_MOUSE_INIT) : "memory");
}

static inline void rlx_read_mouse(int *state) {
    __asm__ __volatile__("int $0x80" :: "a"(SYS_READ_MOUSE), "D"(state) : "memory");
}

// ============================================================
// УТИЛИТЫ
// ============================================================
static void itoa(unsigned int n, char *buf) {
    if (!n) { buf[0]='0'; buf[1]=0; return; }
    char t[12]; int i=0;
    while (n) { t[i++]='0'+n%10; n/=10; }
    int j=0; while(i>0) buf[j++]=t[--i]; buf[j]=0;
}

static int streq(const char *a, const char *b) {
    while(*a && *b) if(*a++!=*b++) return 0;
    return *a==*b;
}

static void strcopy(char *dst, const char *src, int max) {
    int i=0; while(src[i] && i<max-1){dst[i]=src[i];i++;} dst[i]=0;
}

// ============================================================
// ОТРИСОВКА КУРСОРА МЫШИ
// ============================================================
static char  prev_char = ' ';
static unsigned char prev_attr = 0x07;
static int   prev_mx = -1, prev_my = -1;

static void draw_mouse_cursor(void) {
    // Восстановить предыдущую ячейку
    if (prev_mx >= 0)
        VGA[prev_my * VGA_W + prev_mx] = (unsigned short)(unsigned char)prev_char | ((unsigned short)prev_attr << 8);

    if (mouse_x < 0 || mouse_x >= VGA_W || mouse_y < 0 || mouse_y >= VGA_H) return;

    // Сохранить текущую ячейку
    unsigned short cell = VGA[mouse_y * VGA_W + mouse_x];
    prev_char = (char)(cell & 0xFF);
    prev_attr = (unsigned char)(cell >> 8);
    prev_mx = mouse_x; prev_my = mouse_y;

    // Рисуем курсор — инверсия цветов
    unsigned char cursor_fg = (prev_attr >> 4) & 0xF;
    unsigned char cursor_bg = prev_attr & 0xF;
    VGA[mouse_y * VGA_W + mouse_x] = vga_cell(prev_char ? prev_char : ' ', cursor_bg, cursor_fg);
}

// ============================================================
// ОПРОС МЫШИ
// ============================================================
static void poll_mouse(void) {
    int state[3] = {0, 0, 0};
    rlx_read_mouse(state);
    mouse_x = state[0];
    mouse_y = state[1];
    mouse_prev_lbtn = mouse_lbtn;
    mouse_lbtn = (state[2] & 0x01);
}

// Проверить кнопку — свежий клик (нажали, потом отпустили)
static int mouse_click_at(int x, int y, int w, int h) {
    return mouse_prev_lbtn && !mouse_lbtn &&
           mouse_x >= x && mouse_x < x+w &&
           mouse_y >= y && mouse_y < y+h;
}

// ============================================================
// ПАНЕЛЬ ЗАДАЧ (строка 0)
// ============================================================
typedef struct { const char *label; int x; } TabEntry;
static const TabEntry TABS[4] = {
    {" Desktop ", 13},
    {" Terminal ", 23},
    {" SysInfo ", 34},
    {" Files ", 44},
};

static void draw_taskbar(void) {
    vfill(0, 0, VGA_W, 1, ' ', C_WHITE, C_BLUE);

    // Логотип
    vprint(0, 0, " [", C_YELLOW, C_BLUE);
    vput(2, 0, 'R', C_LCYAN, C_BLUE);
    vprint(3, 0, "ealix", C_WHITE, C_BLUE);
    vprint(8, 0, " DE]", C_YELLOW, C_BLUE);

    // Вкладки
    for (int i = 0; i < 4; i++) {
        unsigned char bg = (desktop_state == i) ? C_CYAN  : C_DGRAY;
        unsigned char fg = (desktop_state == i) ? C_BLACK : C_WHITE;
        // Подсвечивать если мышь над вкладкой
        int tx = TABS[i].x, tw = vstrlen(TABS[i].label);
        if (mouse_y == 0 && mouse_x >= tx && mouse_x < tx + tw)
            bg = (desktop_state == i) ? C_LCYAN : C_LGRAY;
        vprint(tx, 0, TABS[i].label, fg, bg);
    }

    // Аптайм
    char up[12]; itoa(uptime, up);
    vprint(54, 0, "UP:", C_LGRAY,  C_BLUE);
    vprint(57, 0, up,   C_YELLOW, C_BLUE);
    vprint(57+vstrlen(up), 0, "s", C_LGRAY, C_BLUE);

    // Подсказки
    vprint(65, 0, " Q:Exit", C_LGRAY, C_BLUE);

    // Нижняя статус-строка (строка 24)
    vfill(0, 24, VGA_W, 1, ' ', C_LGRAY, C_DGRAY);
    char pos[24] = "Mouse: (  ,  )  L:";
    pos[8]  = '0' + (mouse_x / 10);
    pos[9]  = '0' + (mouse_x % 10);
    pos[11] = '0' + (mouse_y / 10);
    pos[12] = '0' + (mouse_y % 10);
    pos[18] = mouse_lbtn ? '1' : '0';
    pos[19] = 0;
    vprint(1, 24, pos, C_LGRAY, C_DGRAY);
    vprint(55, 24, "Realix Desktop 32-bit Ring 3", C_DGRAY, C_DGRAY);
}

// ============================================================
// РАБОЧИЙ СТОЛ
// ============================================================
typedef struct { const char *title; const char *sub; int x; int y; int target; } Icon;
static const Icon ICONS[8] = {
    { "Terminal",  "CLI Shell",   2,  2, S_TERMINAL },
    { "SysInfo",   "CPU & RAM",  18,  2, S_SYSINFO  },
    { "Files",     "FAT12 Mgr",  34,  2, S_FILES    },
    { "RlxFetch",  "OS Info",    50,  2, S_SYSINFO  },
    { "Snake",     "Arcade",      2,  8, -1          },
    { "Exec16",    "16bit Mode", 18,  8, -1          },
    { "Settings",  "Prefs",      34,  8, -1          },
    { "Reboot",    "Restart",    50,  8, -1          },
};

static void draw_icon(const Icon *ic, int hover) {
    int x = ic->x, y = ic->y + 1; // +1 из-за тасбара
    unsigned char border_fg = hover ? C_YELLOW : C_LCYAN;
    unsigned char text_bg   = hover ? C_DGRAY  : C_BLUE;

    vput(x,    y,   0xDA, border_fg, text_bg);
    for(int i=1;i<13;i++) vput(x+i, y,   0xC4, border_fg, text_bg);
    vput(x+13, y,   0xBF, border_fg, text_bg);

    vput(x,    y+1, 0xB3, border_fg, text_bg);
    vprint(x+1, y+1, ic->title, hover ? C_YELLOW : C_WHITE, text_bg);
    for(int i=x+1+vstrlen(ic->title);i<x+13;i++) vput(i, y+1, ' ', C_WHITE, text_bg);
    vput(x+13, y+1, 0xB3, border_fg, text_bg);

    vput(x,    y+2, 0xB3, border_fg, text_bg);
    vprint(x+1, y+2, ic->sub, C_LGRAY, text_bg);
    for(int i=x+1+vstrlen(ic->sub);i<x+13;i++) vput(i, y+2, ' ', C_WHITE, text_bg);
    vput(x+13, y+2, 0xB3, border_fg, text_bg);

    vput(x,    y+3, 0xC0, border_fg, text_bg);
    for(int i=1;i<13;i++) vput(x+i, y+3, 0xC4, border_fg, text_bg);
    vput(x+13, y+3, 0xD9, border_fg, text_bg);
}

static void draw_desktop(void) {
    vfill(0, 1, VGA_W, VGA_H-2, ' ', C_LGRAY, C_BLUE);

    vcenter(2, VGA_W, "=== Realix Desktop Environment v2.0 ===", C_YELLOW, C_BLUE);
    vcenter(3, VGA_W, "32-bit Protected Mode | Ring 3 | PS/2 Mouse Enabled", C_LGRAY, C_BLUE);

    for (int i = 0; i < 8; i++) {
        // Проверяем ховер мыши над иконкой
        int ix = ICONS[i].x, iy = ICONS[i].y + 1 + 1; // y +1 taskbar +1 header
        int hover = (mouse_x >= ix && mouse_x < ix+14 &&
                     mouse_y >= iy && mouse_y < iy+4);
        draw_icon(&ICONS[i], hover);
    }

    vprint(14, 21, "[1-4] Top Row  [5-8] Bottom Row  [Keyboard or Click]", C_YELLOW, C_BLUE);
    vprint(20, 22, "[ESC/Click Tab] Switch Window   [Q] Quit", C_LGRAY, C_BLUE);
}

// ============================================================
// ТЕРМИНАЛ
// ============================================================
static void term_push(const char *line) {
    if (term_line_count >= 10) {
        for(int i=0;i<9;i++) strcopy(term_lines[i], term_lines[i+1], 80);
        term_line_count = 9;
    }
    strcopy(term_lines[term_line_count++], line, 80);
}

static void term_run(void) {
    // Добавить строку ввода в историю
    char prompt[80];
    const char *pfx = ">> ";
    int pi = 0;
    while(pfx[pi]) prompt[pi] = pfx[pi++];
    for(int i=0;i<term_input_len;i++) prompt[pi++] = term_input[i];
    prompt[pi] = 0;
    term_push(prompt);

    // Обработка команд
    if (streq(term_input, "help"))        term_push("  help ver clear uptime sysinfo echo <text>");
    else if (streq(term_input, "ver"))    term_push("  Realix OS v0.1 (32-bit Desktop32 v2.0)");
    else if (streq(term_input, "clear")) { term_line_count = 0; }
    else if (streq(term_input, "uptime")) {
        char buf[40] = "  Uptime: "; char n[12]; itoa(uptime, n);
        int i = 10; char *np = n;
        while(*np) buf[i++]=*np++;
        buf[i++]=' '; buf[i++]='s'; buf[i]=0;
        term_push(buf);
    }
    else if (streq(term_input, "sysinfo")) {
        term_push("  OS: Realix v0.1  Kernel: kernel32 (Rust+NASM)");
        term_push("  Mode: 32-bit Protected  Ring: 3  GDT: 16");
        term_push("  Mouse: PS/2 IRQ12  Keyboard: PS/2 IRQ1");
    }
    else if (term_input[0]=='e' && term_input[1]=='c' && term_input[2]=='h' && term_input[3]=='o' && term_input[4]==' ') {
        char out[80] = "  "; int i=2; char *src=term_input+5; while(*src) out[i++]=*src++; out[i]=0;
        term_push(out);
    }
    else if (term_input[0] != 0) {
        char err[64] = "  Unknown: "; int i=11; char *s=term_input; while(*s) err[i++]=*s++; err[i]=0;
        term_push(err);
    }

    term_input_len = 0; term_input[0] = 0;
}

static void draw_terminal(void) {
    vfill(0, 1, VGA_W, VGA_H-2, ' ', C_LGREEN, C_BLACK);

    // Заголовок окна
    vfill(0, 1, VGA_W, 1, ' ', C_WHITE, C_DGRAY);
    vprint(1, 1, "[ TERMINAL ] Realix Shell  |  32-bit Ring 3 Process  |  [ESC] Close", C_LGRAY, C_DGRAY);

    // История вывода
    int ybase = 2;
    if (term_line_count == 0) {
        vprint(1, 2,  " Realix Terminal v2.0 — PS/2 Mouse & Keyboard Ready", C_YELLOW, C_BLACK);
        vprint(1, 3,  " Type 'help' for available commands", C_LGRAY, C_BLACK);
        vprint(1, 4,  " ─────────────────────────────────────────────────", C_DGRAY, C_BLACK);
        ybase = 5;
    } else {
        int start = term_line_count > 16 ? term_line_count - 16 : 0;
        for (int i = start; i < term_line_count; i++) {
            unsigned char fg = (term_lines[i][0]=='>') ? C_LCYAN : C_LGREEN;
            vprint(1, ybase++, term_lines[i], fg, C_BLACK);
        }
    }

    // Строка ввода
    vfill(0, 22, VGA_W, 1, ' ', C_WHITE, C_BLACK);
    vprint(0, 22, "Realix >> ", C_LCYAN, C_BLACK);
    vprint(10, 22, term_input, C_WHITE, C_BLACK);
    // Курсор мигающий
    vput(10 + term_input_len, 22, '_', C_YELLOW, C_BLACK);

    // Подсказка
    vfill(0, 23, VGA_W, 1, ' ', C_DGRAY, C_BLACK);
    vprint(1, 23, "[ENTER] Run  [BACKSPACE] Del  [ESC] Back to Desktop", C_DGRAY, C_BLACK);
}

// ============================================================
// СИСТЕМНАЯ ИНФОРМАЦИЯ
// ============================================================
static void draw_sysinfo(void) {
    vfill(0, 1, VGA_W, VGA_H-2, ' ', C_LGRAY, C_BLUE);
    vfill(0, 1, VGA_W, 1, ' ', C_WHITE, C_DGRAY);
    vprint(1, 1, "[ SYSTEM INFO ]  Realix Hardware & Kernel Report  |  [ESC] Back", C_LGRAY, C_DGRAY);

    // ASCII лого R
    static const char *logo[] = {
        "RRRRRRRRRRRRRR",
        "RR           RR",
        "RR           RR",
        "RRRRRRRRRRRRRR",
        "RR       RR",
        "RR         RR",
        "RR           RR",
    };
    for (int i = 0; i < 7; i++) {
        const char *l = logo[i]; int x = 3;
        while (*l) {
            vput(x++, 3+i, *l, (*l=='R') ? C_LBLUE : C_BLUE, C_BLUE);
            l++;
        }
    }

    // Данные системы
    vprint(22, 3,  "User @ realix-desktop", C_WHITE,  C_BLUE);
    vprint(22, 4,  "─────────────────────", C_DGRAY,  C_BLUE);
    vprint(22, 5,  "OS:      Realix OS v0.1",            C_WHITE,  C_BLUE);
    vprint(22, 6,  "Kernel:  kernel32 (Rust + NASM)",    C_WHITE,  C_BLUE);
    vprint(22, 7,  "Desktop: Desktop32 v2.0",            C_WHITE,  C_BLUE);
    vprint(22, 8,  "Mode:    32-bit Protected Mode",     C_WHITE,  C_BLUE);
    vprint(22, 9,  "Ring:    Ring 3 (User Mode)",        C_LGREEN, C_BLUE);
    vprint(22, 10, "API:     INT 0x80 (SYS 1-9)",        C_LCYAN,  C_BLUE);
    vprint(22, 11, "GDT:     16 Descriptors",            C_WHITE,  C_BLUE);
    vprint(22, 12, "TSS:     Ring0 Stack @ 0x88000",     C_WHITE,  C_BLUE);
    vprint(22, 13, "Mouse:   PS/2 IRQ12 Enabled",        C_YELLOW, C_BLUE);
    vprint(22, 14, "Kbd:     PS/2 IRQ1 Scancode Set 1",  C_WHITE,  C_BLUE);
    vprint(22, 15, "PATH:    /bin;/apps;/",              C_LGREEN, C_BLUE);

    // Аптайм
    char upbuf[40] = "Uptime:  ";
    char unum[12]; itoa(uptime, unum);
    int i=9; char *n=unum; while(*n) upbuf[i++]=*n++;
    upbuf[i++]=' '; upbuf[i++]='s'; upbuf[i]=0;
    vprint(22, 16, upbuf, C_YELLOW, C_BLUE);

    // Позиция мыши
    char mpos[40] = "Mouse:   X=  Y=";
    mpos[9]  = '0' + (mouse_x / 10); mpos[10] = '0' + (mouse_x % 10);
    mpos[13] = '0' + (mouse_y / 10); mpos[14] = '0' + (mouse_y % 10);
    mpos[15] = 0;
    vprint(22, 17, mpos, C_LCYAN, C_BLUE);
}

// ============================================================
// ФАЙЛОВЫЙ МЕНЕДЖЕР
// ============================================================
static void draw_files(void) {
    vfill(0, 1, VGA_W, VGA_H-2, ' ', C_LGRAY, C_BLACK);
    vfill(0, 1, VGA_W, 1, ' ', C_WHITE, C_DGRAY);
    vprint(1, 1, "[ FILES ]  Realix FAT12 Virtual Filesystem  |  [ESC] Back", C_LGRAY, C_DGRAY);

    // Заголовок
    vfill(0, 2, VGA_W, 1, ' ', C_YELLOW, C_DGRAY);
    vprint(1,  2, "Name                Size    Type          Description", C_YELLOW, C_DGRAY);

    typedef struct { const char *n; const char *sz; const char *t; const char *d; } FEntry;
    static const FEntry F[] = {
        {"bootix.bin",     "512B",  "[BOOTLOADER]", "Stage 1 MBR — 512b x86 NASM"},
        {"initrix.bin",    "4KB",   "[BOOTLOADER]", "Stage 2 Loader — FAT12 + PM switch"},
        {"kernel16.bin",   "24KB",  "[KERNEL-16]",  "Real Mode Kernel (NASM)"},
        {"kernel32.bin",   "64KB",  "[KERNEL-32]",  "Protected Mode Kernel (Rust nightly)"},
        {"app16.rlx",      "2KB",   "[RLX/16]",     "Demo 16-bit Ring 3 App"},
        {"app32.rlx",      "2KB",   "[RLX/32]",     "Demo 32-bit Ring 3 App"},
        {"snake32.rlx",    "8KB",   "[RLX/32]",     "Snake Arcade Game (Ring 3 NASM)"},
        {"rlxfetch.rlx",   "12KB",  "[RLX/32]",     "FastFetch Port — C11 GCC-15"},
        {"rlxfetch16.rlx", "4KB",   "[RLX/16]",     "SysFetch 16-bit (NASM, BIOS INT 10h)"},
        {"exec16.rlx",     "4KB",   "[RLX/32]",     "32→16-bit Subsystem Bridge (C11)"},
        {"desktop32.rlx",  "20KB",  "[RLX/32]",     "Desktop32 WM v2.0 — Mouse + Keyboard"},
    };

    for (int i = 0; i < 11; i++) {
        unsigned char bg = C_BLACK;
        int hover = (mouse_y == 3+i && mouse_x >= 0 && mouse_x < VGA_W);
        if (hover) bg = C_DGRAY;
        vfill(0, 3+i, VGA_W, 1, ' ', C_LGRAY, bg);
        vprint(1,  3+i, F[i].n,  C_LGREEN, bg);
        vprint(21, 3+i, F[i].sz, C_YELLOW, bg);
        vprint(29, 3+i, F[i].t,  C_LCYAN,  bg);
        vprint(43, 3+i, F[i].d,  C_LGRAY,  bg);
    }

    vfill(0, 23, VGA_W, 1, ' ', C_DGRAY, C_BLACK);
    vprint(1, 23, "Hover row to highlight  |  11 files in FAT12 root  |  1.44 MB total", C_DGRAY, C_BLACK);
}

// ============================================================
// ГЛАВНЫЙ ЦИКЛ
// ============================================================
int main(void) {
    // Инициализация мыши
    rlx_mouse_init();

    // Первоначальная отрисовка
    vfill(0, 0, VGA_W, VGA_H, ' ', C_LGRAY, C_BLUE);
    draw_taskbar();
    draw_desktop();
    draw_mouse_cursor();

    static int   mouse_state[3];
    static int   prev_state  = -1;   // Предыдущее состояние экрана
    static int   prev_mx     = -1;   // Предыдущая позиция курсора X
    static int   prev_my     = -1;   // Предыдущая позиция курсора Y
    static unsigned int loop_counter = 0;

    while (1) {
        loop_counter++;

        // ------ Опрос мыши (неблокирующий, каждую итерацию) ------
        rlx_read_mouse(mouse_state);
        int new_mx = mouse_state[0];
        int new_my = mouse_state[1];
        mouse_prev_lbtn = mouse_lbtn;
        mouse_lbtn = (mouse_state[2] & 1);

        // ------ Неблокирующее чтение клавиши ------
        unsigned int k = rlx_read_key_nb();  // 0 = нет нажатия

        // Аптайм каждые ~2048 итераций
        if ((loop_counter & 0x7FF) == 0) uptime++;

        // ------ Флаг перерисовки — только при реальных изменениях ------
        int needs_redraw = 0;

        // Изменилась позиция мыши?
        if (new_mx != prev_mx || new_my != prev_my) {
            mouse_x = new_mx; mouse_y = new_my;
            prev_mx = new_mx; prev_my = new_my;
            needs_redraw = 1;
        }

        // Изменилось состояние кнопки мыши?
        if (mouse_prev_lbtn != mouse_lbtn) {
            needs_redraw = 1;
        }

        // ------ Клики мышью по вкладкам ------
        if (mouse_click_at(TABS[0].x, 0, 9,  1)) { desktop_state = S_DESKTOP;  needs_redraw = 1; }
        if (mouse_click_at(TABS[1].x, 0, 10, 1)) { desktop_state = S_TERMINAL; needs_redraw = 1; }
        if (mouse_click_at(TABS[2].x, 0, 9,  1)) { desktop_state = S_SYSINFO;  needs_redraw = 1; }
        if (mouse_click_at(TABS[3].x, 0, 7,  1)) { desktop_state = S_FILES;    needs_redraw = 1; }

        // ------ Клики по иконкам рабочего стола ------
        if (desktop_state == S_DESKTOP) {
            for (int i = 0; i < 8; i++) {
                int ix = ICONS[i].x, iy = ICONS[i].y + 1 + 1;
                if (mouse_click_at(ix, iy, 14, 4) && ICONS[i].target >= 0) {
                    desktop_state = ICONS[i].target;
                    needs_redraw = 1;
                }
            }
        }

        // ------ Обработка клавиш ------
        if (k != 0) {
            needs_redraw = 1;

            // Выход
            if (k == 'q' || k == 'Q') {
                vfill(0, 0, VGA_W, VGA_H, ' ', C_WHITE, C_BLACK);
                vprint(22, 12, "Exiting Desktop Environment...", C_YELLOW, C_BLACK);
                break;
            }

            // F1-F4 — переключение режимов (глобально, даже из терминала)
            if (k == KEY_F1) desktop_state = S_DESKTOP;
            if (k == KEY_F2) desktop_state = S_TERMINAL;
            if (k == KEY_F3) desktop_state = S_SYSINFO;
            if (k == KEY_F4) desktop_state = S_FILES;

            // ESC — вернуться на рабочий стол
            if (k == KEY_ESC) desktop_state = S_DESKTOP;

            // Цифры вне терминала
            if (desktop_state != S_TERMINAL) {
                if      (k == '1' || k == 't') desktop_state = S_TERMINAL;
                else if (k == '2' || k == 'i') desktop_state = S_SYSINFO;
                else if (k == '3' || k == 'f') desktop_state = S_FILES;
                else if (k == '5')             desktop_state = S_DESKTOP;
                if (k == KEY_UP   && desktop_state > 0) desktop_state--;
                if (k == KEY_DOWN && desktop_state < 3) desktop_state++;
            }

            // Ввод в терминале
            if (desktop_state == S_TERMINAL) {
                if (k == KEY_ESC) {
                    desktop_state = S_DESKTOP;
                } else if (k == KEY_ENTER || k == '\r') {
                    term_run();
                } else if (k == KEY_BACKSPACE) {
                    if (term_input_len > 0) term_input[--term_input_len] = 0;
                } else if (k >= 32 && k < 127 && term_input_len < 62) {
                    term_input[term_input_len++] = (char)k;
                    term_input[term_input_len] = 0;
                }
            }
        }

        // ------ Перерисовка ТОЛЬКО при изменениях (устраняет мерцание) ------
        if (needs_redraw || desktop_state != prev_state) {
            prev_state = desktop_state;
            draw_taskbar();
            switch (desktop_state) {
                case S_DESKTOP:  draw_desktop();  break;
                case S_TERMINAL: draw_terminal(); break;
                case S_SYSINFO:  draw_sysinfo();  break;
                case S_FILES:    draw_files();    break;
            }
            // Курсор мыши поверх всего содержимого
            draw_mouse_cursor();
        }
    }

    rlx_exit();
    return 0;
}
