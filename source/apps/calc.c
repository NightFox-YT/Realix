// © Realix > calc.rlx — Interactive Expression Calculator
// (28.07.26) v1.0  — Ring 3, 32-bit, no libc
// ====================================================
// Recursive descent parser — поддержка:
//   + - * / %  скобки ()  унарный минус  целые числа
//
// Грамматика:
//   expr   = term   { ('+' | '-') term   }
//   term   = factor { ('*' | '/' | '%') factor }
//   factor = ['-'] ( '(' expr ')' | NUMBER )
// ====================================================

#define SYS_EXIT        3
#define SYS_READ_KEY    4
#define SYS_PUTCHAR     2

// ──────────────────────────────────────────────────────
// Системные вызовы
// ──────────────────────────────────────────────────────
static inline void rlx_exit(void) {
    __asm__ __volatile__("int $0x80" :: "a"(SYS_EXIT) : "memory");
}

static inline unsigned int rlx_read_key(void) {
    unsigned int r;
    __asm__ __volatile__("int $0x80" : "=a"(r) : "a"(SYS_READ_KEY) : "memory");
    return r;
}

static inline void rlx_putchar(char c, unsigned char fg, unsigned char bg) {
    __asm__ __volatile__(
        "int $0x80"
        :: "a"(SYS_PUTCHAR), "b"((unsigned int)c),
           "c"((unsigned int)fg), "d"((unsigned int)bg)
        : "memory"
    );
}

// ──────────────────────────────────────────────────────
// VGA Terminal Output
// ──────────────────────────────────────────────────────
#define FG_WHITE   15
#define FG_CYAN    11
#define FG_GREEN   10
#define FG_YELLOW  14
#define FG_RED     12
#define FG_LGRAY    7
#define FG_MAGENTA 13
#define BG_BLACK    0

static void print_char(char c, unsigned char fg) {
    rlx_putchar(c, fg, BG_BLACK);
}

static void print_str(const char *s, unsigned char fg) {
    while (*s) print_char(*s++, fg);
}

static void print_nl(void) {
    print_char('\n', FG_WHITE);
}

static void print_int(int n, unsigned char fg) {
    if (n == 0) { print_char('0', fg); return; }

    char buf[12];
    int i = 0;
    int neg = 0;

    if (n < 0) { neg = 1; n = -n; }

    while (n > 0) {
        buf[i++] = '0' + (n % 10);
        n /= 10;
    }
    if (neg) buf[i++] = '-';

    while (i > 0) print_char(buf[--i], fg);
}

static int str_starts(const char *s, const char *prefix) {
    while (*prefix) { if (*s++ != *prefix++) return 0; }
    return 1;
}

static int str_eq(const char *a, const char *b) {
    while (*a && *b) { if (*a++ != *b++) return 0; }
    return *a == *b;
}

// ──────────────────────────────────────────────────────
// Ввод строки
// ──────────────────────────────────────────────────────
static char line[128];
static int  line_len = 0;

static void read_line(void) {
    line_len = 0;
    line[0]  = 0;

    while (1) {
        unsigned int k = rlx_read_key();

        if (k == '\n' || k == '\r' || k == 0x0D) {
            print_nl();
            break;
        } else if (k == 0x08 || k == 0x7F) {          // Backspace
            if (line_len > 0) {
                line_len--;
                line[line_len] = 0;
                // Визуальный откат (backspace + space + backspace)
                print_char('\x08', FG_WHITE);
                print_char(' ',    FG_WHITE);
                print_char('\x08', FG_WHITE);
            }
        } else if (k >= 32 && k < 127 && line_len < 126) {
            line[line_len++] = (char)k;
            line[line_len]   = 0;
            print_char((char)k, FG_WHITE);
        }
    }
}

// ──────────────────────────────────────────────────────
// Парсер выражений (Recursive Descent)
// ──────────────────────────────────────────────────────
static const char *pos;   // Текущая позиция в строке
static int parse_ok;      // Флаг успешного разбора
static char err_msg[48];  // Сообщение об ошибке

static void set_error(const char *msg) {
    parse_ok = 0;
    const char *s = msg;
    char *d = err_msg;
    while (*s) *d++ = *s++;
    *d = 0;
}

// Пропустить пробелы
static void skip_ws(void) {
    while (*pos == ' ' || *pos == '\t') pos++;
}

// Прочитать целое число
static int read_number(void) {
    skip_ws();
    if (*pos < '0' || *pos > '9') {
        set_error("Expected number");
        return 0;
    }
    int n = 0;
    while (*pos >= '0' && *pos <= '9') {
        n = n * 10 + (*pos - '0');
        pos++;
    }
    return n;
}

// Предварительные объявления
static int parse_expr(void);
static int parse_term(void);
static int parse_factor(void);

// factor = ['-'] ( '(' expr ')' | NUMBER )
static int parse_factor(void) {
    skip_ws();
    if (!parse_ok) return 0;

    int negative = 0;
    while (*pos == '-') {
        negative = !negative;
        pos++;
        skip_ws();
    }

    int val = 0;

    if (*pos == '(') {
        pos++;
        val = parse_expr();
        skip_ws();
        if (*pos == ')') {
            pos++;
        } else if (parse_ok) {
            set_error("Expected ')'");
        }
    } else {
        val = read_number();
    }

    return negative ? -val : val;
}

// term = factor { ('*' | '/' | '%') factor }
static int parse_term(void) {
    int left = parse_factor();
    if (!parse_ok) return 0;

    while (1) {
        skip_ws();
        char op = *pos;
        if (op != '*' && op != '/' && op != '%') break;
        pos++;

        int right = parse_factor();
        if (!parse_ok) return 0;

        if (op == '*') {
            left *= right;
        } else if (op == '/') {
            if (right == 0) { set_error("Division by zero"); return 0; }
            left /= right;
        } else {
            if (right == 0) { set_error("Division by zero"); return 0; }
            left %= right;
        }
    }
    return left;
}

// expr = term { ('+' | '-') term }
static int parse_expr(void) {
    int left = parse_term();
    if (!parse_ok) return 0;

    while (1) {
        skip_ws();
        char op = *pos;
        if (op != '+' && op != '-') break;
        pos++;

        int right = parse_term();
        if (!parse_ok) return 0;

        if (op == '+') left += right;
        else           left -= right;
    }
    return left;
}

// Главная функция вычисления
static int evaluate(const char *expr, int *ok) {
    pos      = expr;
    parse_ok = 1;
    err_msg[0] = 0;

    int result = parse_expr();

    skip_ws();
    if (parse_ok && *pos != 0) {
        set_error("Unexpected character");
        parse_ok = 0;
    }

    *ok = parse_ok;
    return result;
}

// ──────────────────────────────────────────────────────
// Отрисовка шапки
// ──────────────────────────────────────────────────────
static void print_banner(void) {
    print_nl();
    print_str("  ╔══════════════════════════════════════╗", FG_CYAN);
    print_nl();
    print_str("  ║   calc.rlx  —  Realix Calculator     ║", FG_CYAN);
    print_nl();
    print_str("  ║   Ring 3  |  32-bit  |  v1.0          ║", FG_CYAN);
    print_nl();
    print_str("  ╚══════════════════════════════════════╝", FG_CYAN);
    print_nl();
    print_nl();
    print_str("  Operators: ", FG_LGRAY);
    print_str("+ - * / %", FG_YELLOW);
    print_str("   Grouping: ", FG_LGRAY);
    print_str("( )", FG_YELLOW);
    print_str("   Unary: ", FG_LGRAY);
    print_str("-", FG_YELLOW);
    print_nl();
    print_str("  Type expression and press ENTER. ", FG_LGRAY);
    print_str("'quit' or 'q' to exit.", FG_LGRAY);
    print_nl();
    print_nl();
}

// ──────────────────────────────────────────────────────
// Главная точка входа
// ──────────────────────────────────────────────────────
int main(void) {
    print_banner();

    while (1) {
        // Prompt
        print_str("  calc", FG_CYAN);
        print_str("> ", FG_YELLOW);

        // Читаем строку ввода
        read_line();

        // Пропустить пустой ввод
        if (line_len == 0) continue;

        // Проверить команды выхода
        if (str_eq(line, "quit") || str_eq(line, "q") ||
            str_eq(line, "exit") || str_eq(line, ":q")) {
            print_nl();
            print_str("  Bye!", FG_CYAN);
            print_nl();
            print_nl();
            break;
        }

        // Проверить команду help
        if (str_eq(line, "help") || str_eq(line, "?")) {
            print_nl();
            print_str("  Supported operators:", FG_LGRAY);
            print_nl();
            print_str("    +   Addition       e.g.  5 + 3  = 8", FG_WHITE);
            print_nl();
            print_str("    -   Subtraction    e.g.  9 - 4  = 5", FG_WHITE);
            print_nl();
            print_str("    *   Multiply       e.g.  6 * 7  = 42", FG_WHITE);
            print_nl();
            print_str("    /   Integer div    e.g. 10 / 3  = 3", FG_WHITE);
            print_nl();
            print_str("    %   Modulo         e.g. 10 % 3  = 1", FG_WHITE);
            print_nl();
            print_str("    ()  Grouping       e.g. 2*(3+4) = 14", FG_WHITE);
            print_nl();
            print_str("    -   Unary minus    e.g. -(5+2)  = -7", FG_WHITE);
            print_nl();
            print_nl();
            continue;
        }

        // Вычислить выражение
        int ok = 0;
        int result = evaluate(line, &ok);

        if (ok) {
            // Успех: показать результат
            print_str("  = ", FG_GREEN);
            print_int(result, FG_WHITE);
            print_nl();
            print_nl();
        } else {
            // Ошибка
            print_str("  Error: ", FG_RED);
            print_str(err_msg, FG_RED);
            print_nl();
            // Показать где ошибка стрелкой
            int offset = (int)(pos - line);
            print_str("  ", FG_RED);
            // Напечатать строку до ошибки
            for (int i = 0; i < offset && i < 60; i++)
                print_char(line[i], FG_LGRAY);
            print_str(" <-- here", FG_RED);
            print_nl();
            print_nl();
        }
    }

    rlx_exit();
    return 0;
}
