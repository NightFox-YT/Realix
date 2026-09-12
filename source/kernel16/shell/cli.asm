; © Realix > Shell: Command Line Interface
; ø Inspired by Nyx
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/*, bios-api/keyboard,
;                 kernel16/shell/commands (execute_cmd)
;                 kernel16/shell/history

; Основные константы
%include 'shared/config.asm'

; ASCII коды клавиш
ENTER_KEY     equ 0x0D
BACKSPACE_KEY equ 0x08
ESCAPE_KEY    equ 0x1B
SPACE_KEY     equ 0x20

; Скан-коды расширенных клавиш (int 0x16, al = 0)
KEY_UP_SCAN   equ 0x48
KEY_DOWN_SCAN equ 0x50
KEY_F7_SCAN   equ 0x41

; Настройки CLI
INPUT_BUFFER_LEN  equ 64


; > Главный цикл CLI (Вызывается из ядра)
cli_run:
    push si

.prompt:
    ; "Realix >> "
    mov si, prompt_sign
    call print

    ; Ожидание ввода строки
    call cli_input

    ; Если ввод пустой, заново ждём ввод
    cmp byte [input_buffer], 0
    je .prompt

    ; Выполнение команды
    mov si, input_buffer
    call execute_cmd

    ; Перевод строки, если команда не перевела сама
    call print_new_line_if_needed

    jmp .prompt

.return:
    pop si
    ret


; > Функция чтения строки
cli_input:
    push ax
    push bx
    push di

    ; Счётчик введённых символов и указатель адреса буффера
    xor bx, bx
    mov di, input_buffer

    ; Сброс навигации по истории команд
    call history_init

.input_loop:
    ; Ожидание нажатия (ASCII > al)
    call wait_key

    ; ENTER > Проверка введённой строки
    cmp al, ENTER_KEY
    je .enter_pressed

    ; Backspace > Стирание последнего символа
    cmp al, BACKSPACE_KEY
    je .backspace_pressed

    ; Escape > Стирание всей строки ввода
    cmp al, ESCAPE_KEY
    je .escape_pressed

    ; "Расширенные клавиши" > Навигация по истории
    test al, al
    jz .extended_key

    ; Игнор управляющих символов
    cmp al, SPACE_KEY
    jb .input_loop

    ; Проверка на переполнение
    cmp bx, INPUT_BUFFER_LEN
    jae .buffer_full

    ; Отображение символа на экране (Эхо)
    call print_char

    ; Сохранение символа в буфер
    mov [di], al
    inc di
    inc bx

    ; Возвращаемся в поток ввода
    jmp .input_loop

.buffer_full:
    ; Буфер полон: звуковой сигнал
    call print_beep_char
    jmp .input_loop

.extended_key:
    ; Проверка на скан-код стрелок
    cmp ah, KEY_UP_SCAN     ; Стрелка вверх
    je .history_up_arrow
    cmp ah, KEY_DOWN_SCAN   ; Стрелка вниз
    je .history_down_arrow

    ; Проверка на скан-коды функциональных клавиш
    cmp ah, KEY_F7_SCAN
    je .history_list

    ; Прочие "расширенные" клавиши игнорируем
    jmp .input_loop

.history_up_arrow:
    call history_up
    jmp .input_loop

.history_down_arrow:
    call history_down
    jmp .input_loop

.history_list:
    ; Перевод строки и вывод списка истории команд
    call print_new_line
    call history_show

    ; Выводим промпт
    mov si, prompt_sign
    call print

    ; Закрываем текущую строку нуль-терминатором и выводим её
    mov byte [di], 0
    mov si, input_buffer
    call print

    ; Возвращаемся в поток ввода
    jmp .input_loop

.backspace_pressed:
    ; Проверка на пустой буфер
    test bx, bx
    jz .input_loop

    ; Стирание последнего символа из буфера
    dec di
    dec bx
    mov byte [di], 0

    ; Визуальное стирание символа на экране
    call visual_erase_char

    ; Возвращаемся в поток ввода
    jmp .input_loop

.escape_pressed:
    ; Стираем строку ввода
    call erase_input_line

    ; Возвращаемся в поток ввода
    jmp .input_loop

.enter_pressed:
    call print_new_line

    mov byte [di], 0  ; Закрываем строку нуль-терминатором
    call history_add  ; Сохраняем команду в историю

    pop di
    pop bx
    pop ax
    ret


; > Визуальное стирание одного символа на экране
visual_erase_char:
    push ax

    ; Печатаем "Backspace + Space + Backspace"
    mov al, BACKSPACE_KEY
    call print_char
    mov al, ' '
    call print_char
    mov al, BACKSPACE_KEY
    call print_char

    pop ax
    ret

; > Замена видимой строки ввода строкой из si
; Вход:
;  - si: новая строка (0-terminated)
;  - bx: текущая длина строки
;  - di: текущий конец буфера
; Вывод:
;  - bx, di: обновлены под новую строку
cli_replace_line:
    push ax

    ; Стираем текущую строку
    call erase_input_line

; Вывод строки посимвольно на экран
.copy:
    ; Загрузка символа из si в al
    lodsb
    test al, al
    jz .done

    ; Эхо символа на экран
    call print_char

    ; Сохранение символа в буфер
    mov [di], al
    inc di
    inc bx
    jmp .copy

.done:
    ; Закрываем строку нуль-терминатором и выходим
    mov byte [di], 0
    pop ax

    ret

; > Стирание текущей видимой строки ввода с экрана и из буфера
; Вход:
;  - bx: длина строки
;  - di: конец буфера
; Вывод:
;  - bx: 0
;  - di: input_buffer
erase_input_line:
    ; Проверка: Текущая строка теперь пустая?
    test bx, bx
    jz .done

    ; Стираем текущий символ строки
    call visual_erase_char
    dec bx
    jmp erase_input_line

.done:
    ; Обновляыем переменные о строке ввода и выходим
    mov di, input_buffer
    mov byte [di], 0
    
    ret


; > Сравнение двух 0-terminated строк
; Вход:
;  - si, di: указатели на строки
; Выход:
;  - ZF: 1 (строки равны), 0 (строки не равны)
str_equal:
    push si
    push di
    push ax

.loop:
    ; Сравнение символов
    mov al, [si]
    cmp al, [di]
    jne .done

    ; Конец обеих строк (ZF - 1)
    test al, al
    jz .done

    ; Возвращаемся в цикл
    inc si
    inc di
    jmp .loop

.done:
    pop ax
    pop di
    pop si
    ret

; Сообщения и строки
prompt_sign:    db 'Realix >> ', 0
list_separator: db ': ', 0

; Переменные
input_buffer: times (INPUT_BUFFER_LEN + 1) db 0
