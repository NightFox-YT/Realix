; © Realix > Shell: Commands History
; (16.08.26) v0.12
; ================
; ❗️ Зависимости: kernel16/shell/cli

; Настройки истории команд (Кол-во слотов должно быть степенью двойки)
HISTORY_SIZE      equ 16
HISTORY_SLOT_SIZE equ INPUT_BUFFER_LEN + 1


; > Сброс навигации по истории команд
history_init:
    mov word [history_browse], 0
    ret


; > Обработка перехода к более старой записи
history_up:
    ; Уже на самой старой записи
    mov ax, [history_browse]
    cmp ax, [history_count]
    jae .return

    ; Углубляемся на 1 команду в историю
    inc ax
    mov [history_browse], ax

    ; Загружаем запись истории в строку ввода
    ; (si содержит указатель на запись)
    call history_get_ptr
    call cli_replace_line

.return:
    ret


; > Обработка перехода к более новой записи
history_down:
    ; Уже на текущей строке (0)
    mov ax, [history_browse]
    test ax, ax
    jz .return

    ; Выходим на 1 команду обратно
    dec ax
    mov [history_browse], ax

    ; Если пришли к текущей строке
    test ax, ax
    jz .history_down_clear

    ; Загружаем запись истории в строку ввода
    ; (si содержит указатель на запись)
    call history_get_ptr
    call cli_replace_line

.history_down_clear:
    ; Очистка введённой строки
    call erase_input_line

.return:
    ret


; > Добавление непустой строки input_buffer в историю команд
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
    mov dx, HISTORY_SLOT_SIZE
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
    mov dx, HISTORY_SLOT_SIZE
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


; > Указатель на запись истории по текущему `history_browse`
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

    ; Вычисление смещения записи
    ; > slot * (INPUT_BUFFER_LEN + 1)
    mov dx, HISTORY_SLOT_SIZE
    mul dx
    add ax, history_data
    mov si, ax

    pop dx
    pop ax
    ret


; > Вывод буфера истории команд в виде списка
history_show:
    push si
    push ax
    push cx

    ; *Сохраняем `history_browse` (Используется в `history_get_ptr`)
    mov ax, [history_browse]
    push ax

    ; Начальные параметры
    ; (cx - кол-во команд вывода, ax - счётчик)
    mov cx, [history_count]
    xor ax, ax

    ; История пуста: Выводить нечего
    test cx, cx
    jz .done

.print_loop:
    ; Вывод номера след. записи (Увеличивая счётчик)
    inc ax
    call print_dec16

    ; Выводим разделитель для списка
    mov si, list_separator
    call print

    ; Получаем адрес текущей записи (начиная со старой)
    mov [history_browse], cx
    call history_get_ptr

    ; Выводим строку записи
    call print

    ; Переход к след. записи
    call print_new_line
    loop .print_loop

.done:
    ; *Восстановление `history_browse`
    pop ax
    mov [history_browse], ax

    pop cx
    pop ax
    pop si
    ret


; История команд (Кольцевой буфер)
history_data:   times HISTORY_SIZE * HISTORY_SLOT_SIZE db 0
history_next:   dw 0  ; Индекс слота следующей записи
history_count:  dw 0  ; Количество сохранённых команд
history_browse: dw 0  ; Позиция навигации (0 - текущая строка)