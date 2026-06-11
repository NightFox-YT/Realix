volatile char* video_memory = (volatile char*)0xb8000;
int cursor = 0; 
#define BLACK 0x0
#define BLUE 0x1
#define GREEN 0x2
#define CYAN 0x3
#define RED 0x4
#define MAGENTA 0x5
#define BROWN 0x6
#define LIGHT_GRAY 0x7

void putc(char c) {
    if(c == '\n') {
        cursor = ((cursor / 80) + 1) * 80;
        return;
    }
    video_memory[cursor * 2] = c;
    video_memory[cursor * 2 + 1] = LIGHT_GRAY;
    cursor++;
}

void print(char* text) {
    for(int i = 0; text[i] != '\0'; i++) {
        putc(text[i]);
    }
}

void clear_screen() {
    for(int i = 0; i < 80 * 25; i++) {
        video_memory[i * 2] = ' ';
        video_memory[i * 2 + 1] = LIGHT_GRAY;
    }
    cursor = 0;
    }

    void command_terminal(char* command) {
        if(command == "clear") {
            clear_screen();
        }
        else if(command == '\n'){
            print("\n");
        }
        else {
            print("Unknown command: ");
            print(command);
            print("\n");
        }
    }


void kernel_main() {
    clear_screen();
    print("Welcome to Realix Operation System!\nDeveloper : NightFox and Triptolin\n");
}

/* 
                TODO:
                - Реализовать клавиатуру для ввода команд в терминал
                - Командный интерпретатор 
                - Команды: help, info, time
                - Таймер

                ГОТОВО:
                - Команды: clear, \n
                - Ввывод символов 
                - Вывод строк 
                - Перенос строки (\n)
                - Очистка экрана
                - VGA цвета 
*/