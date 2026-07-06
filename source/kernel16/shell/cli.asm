; © Realix > Command Line Interface
; ø Вдохновлено @nyxmalware
; (13.06.26) v0.06
; ================
; ❗️ Зависимости: kernel16/io

; Основные константы
%include 'shared/config.asm'

; > Главный цикл CLI (Вызывается из ядра для обработки команд)
run_cli:
    push si

.prompt:
    ; "Realix >> "
    mov si, prompt_sign
    call print

    call cli_input

    ; Если ввод пустой, заново ждём ввод
    cmp byte [input_buffer], 0
    je .prompt

    ; Выполнение команды
    mov si, input_buffer
    call execute_cmd

    ; Перевод строки на экране
    call print_new_line

    jmp .prompt

.done:
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
    mov word [history_browse], 0

.input_loop:
    ; Ожидание нажатия (ASCII > al)
    xor ah, ah
    int 0x16

    ; ENTER > Проверка введённой строки
    cmp al, ENTER_KEY
    je .enter_pressed
    
    ; Backspace > Стирание последнего символа
    cmp al, BACKSPACE_KEY
    je .backspace_pressed

    ; "Расширенные клавиши" > Навигация по истории
    test al, al
    jz .extended_key

    ; Игнор управляющих символов
    cmp al, 0x20
    jb .input_loop

    ; Проверка на переполнение
    cmp bx, INPUT_BUFFER_LEN
    jae .buffer_full

    ; Отображение символа на экране (Эхо)
    mov ah, 0x0E
    int 0x10

    ; Сохранение символа в буфер
    mov [di], al
    inc di
    inc bx

    ; Возвращаемся в поток ввода
    jmp .input_loop

.backspace_pressed:
    ; Проверка на пустой буффер
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

.buffer_full:
    ; Буфер полон: звуковой сигнал
    call print_beep_char
    jmp .input_loop

.extended_key:
    ; Проверка на скан-код стрелок
    cmp ah, 0x48            ; Стрелка вверх
    je .history_up_arrow
    cmp ah, 0x50            ; Стрелка вниз
    je .history_down_arrow

    ; Прочие "расширенные" клавиши игнорируем
    jmp .input_loop

.history_up_arrow:
    ; Уже на самой старой записи
    mov ax, [history_browse]
    cmp ax, [history_count]
    jae .input_loop

    ; Углубляемся на 1 команду в историю
    inc ax
    mov [history_browse], ax

    ; Загружаем запись истории в строку ввода
    ; si: указатель на запись
    call history_get_ptr
    call history_replace_line
    jmp .input_loop

.history_down_arrow:
    ; Уже на текущей строке (0)
    mov ax, [history_browse]
    test ax, ax
    jz .input_loop

    ; Выходим на 1 команду обратно
    dec ax
    mov [history_browse], ax

    ; Если пришли к текущей строке
    test ax, ax
    jz .history_down_clear

    call history_get_ptr
    call history_replace_line
    jmp .input_loop

.history_down_clear:
    ; Очистка введённой строки
    call erase_input_line
    jmp .input_loop

.enter_pressed:
    call print_new_line

    mov byte [di], 0  ; Закрываем строку нуль-терминатором
    call history_add  ; Сохраняем команду в историю

    pop di
    pop bx
    pop ax
    ret


; > Добавление непустой строки input_buffer в историю команд.
history_add:
    push ax
    push dx
    push si
    push di

    ; Пустую строку не сохраняем
    cmp byte [input_buffer], 0
    je .done

    ; Истории нет, сохраняем сразу
    cmp word [history_count], 0
    je .store

    ; Проверка на дубликат последней команды
    ; > slot = (history_next - 1) & mask
    mov ax, [history_next]
    add ax, HISTORY_SIZE - 1  ; -1 (Защита от ухода в отрицательные числа)
    and ax, HISTORY_SIZE - 1  ; Остаток от деления, т.к. hsize - степень двойки

    ; Высчитываем адрес слота в памяти
    mov dx, INPUT_BUFFER_LEN + 1
    mul dx
    add ax, history_data
    mov di, ax

    ; Проверка на идентичные строки истории и ввода
    mov si, input_buffer
    call str_equal
    je .done

.store:
    ; Вычисляем указатель на слот записи (history_next)
    mov ax, [history_next]
    mov dx, INPUT_BUFFER_LEN + 1
    mul dx
    add ax, history_data
    mov di, ax

    mov si, input_buffer

; Копируем строку ввода в слот истории
.copy:
    lodsb
    mov [di], al
    inc di
    test al, al
    jnz .copy

    ; Обновляем history_next
    ; > (history_next + 1) & mask
    mov ax, [history_next]
    inc ax
    and ax, HISTORY_SIZE - 1
    mov [history_next], ax

    ; Вычисляем новый history_count
    ; > min(history_count + 1, HISTORY_SIZE)
    mov ax, [history_count]
    cmp ax, HISTORY_SIZE
    jae .done
    inc ax
    mov [history_count], ax

.done:
    pop di
    pop si
    pop dx
    pop ax
    ret


; > Указатель на запись истории по текущему history_browse
; Вывод:
;  - si: указатель на строку записи
history_get_ptr:
    push ax
    push dx

    ; Вычисление slot
    ; > (history_next + HISTORY_SIZE - history_browse) & mask
    mov ax, [history_next]
    add ax, HISTORY_SIZE
    sub ax, [history_browse]
    and ax, HISTORY_SIZE - 1

    ; Вычисление смещеняе записи
    ; > slot * (INPUT_BUFFER_LEN + 1)
    mov dx, INPUT_BUFFER_LEN + 1
    mul dx
    add ax, history_data
    mov si, ax

    pop dx
    pop ax
    ret


; > Замена видимой строки ввода строкой из si
; Вход:
;  - si: новая строка (0-terminated)
;  - bx: текущая длина строки
;  - di: текущий конец буфера
; Вывод:
;  - bx, di: обновлены под новую строку
history_replace_line:
    push ax

    ; Стираем текущую строку
    call erase_input_line

; Вывод строки посимвольно на экран
.copy:
    lodsb
    test al, al
    jz .done

    ; Эхо символа на экран
    mov ah, 0x0E
    push bx
    xor bx, bx
    int 0x10
    pop bx

    ; Сохранение символа в буфер
    mov [di], al
    inc di
    inc bx
    jmp .copy

.done:
    mov byte [di], 0
    pop ax
    ret


; > Визуальное стирание одного символа на экране.
visual_erase_char:
    push ax
    push bx

    ; Печатаем "Backspace + Space + Backspace"
    mov ah, 0x0E
    xor bx, bx
    mov al, 0x08
    int 0x10
    mov al, ' '
    int 0x10
    mov al, 0x08
    int 0x10

    pop bx
    pop ax
    ret


; > Стирание текущей видимой строки ввода с экрана и из буфера
; Вход:
;  - bx: длина строки
;  - di: конец буфера
; Вывод:
;  - di: input_buffer
;  - bx: 0
erase_input_line:
    test bx, bx
    jz .done

    call visual_erase_char
    dec bx
    jmp erase_input_line

.done:
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
prompt_sign: db 'Realix >> ', 0

; Переменные
input_buffer: times (INPUT_BUFFER_LEN + 1) db 0

; История команд (Кольцевой буфер)
history_data:   times HISTORY_SIZE * (INPUT_BUFFER_LEN + 1) db 0
history_next:   dw 0  ; Индекс слота следующей записи
history_count:  dw 0  ; Количество сохранённых команд
history_browse: dw 0  ; Позиция навигации (0 - текущая строка)