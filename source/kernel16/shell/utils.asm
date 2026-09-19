; © Realix > Shell: Utils
; (16.08.26) v0.12
; ================

; > Обработка десятичного числа (uint16)
; Параметры:
;  - ds:si: строка с числом
; Вывод:
;  - ax: число (uint16)
;  - si: указывает на след. символ за числом
;  - CF (Carry Flag): 0 (Успех), 1 (Ошибка / Переполнение)
parse_uint16:
    push bx
    push cx
    push dx

    ; Настройка начальный параметров
    xor ax, ax  ; Накопитель результата
    xor cx, cx  ; Счётчик цифр

.next_digit:
    ; Загрузка текущего символа
    xor dx, dx
    mov dl, [si]

    ; Проверка, что ASCII символ является цифрой
    cmp dl, '0'
    jb .finish
    cmp dl, '9'
    ja .finish

    sub dl, '0'  ; ASCII → Цифра (0-9)
    push dx      ; *Сохраняем цифру

    ; Переход к след. разряду числа
    mov bx, 10
    mul bx

    ; Проверка переполнения
    or dx, dx
    pop dx      ; *Восстанавливаем цифру
    jnz .fail   ; Произведение не влезло в 16 битное число

    add ax, dx  ; ax += цифра
    jc .fail    ; Сумма не влезла в 16-битное число

    ; Переход к след. символу (цифре)
    inc si
    inc cx
    jmp .next_digit

.finish:
    ; Проверка на наличие цифр после парсинга
    test cx, cx
    jz .fail

.done:
    clc
    jmp .return

.fail:
    stc

.return:
    pop dx
    pop cx
    pop bx
    ret


; > Перевод символа al в верхний регистр (a-z -> A-Z, иначе без изменений)
; Параметры & Вывод:
;  - al: символ (Любой + a-z)
char_to_upper:
    cmp al, 'a'
    jb .done
    cmp al, 'z'
    ja .done
    sub al, 32
.done:
    ret


; > Перевод символа al в нижний регистр (A-Z -> a-z, иначе без изменений)
; Параметры & Вывод:
;  - al: символ (Любой + A-Z)
char_to_lower:
    cmp al, 'A'
    jb .done
    cmp al, 'Z'
    ja .done
    add al, 32
.done:
    ret


; > Пропуск пробелов до первого символа
; Вывод:
;  - si: указатель на первый символ строки
skip_spaces:
    ; Пропуск пробела c переходом к след. символу
    lodsb
    cmp al, ' '
    je skip_spaces

.done:
    ; Возвращаем si назад на первый символ строки
    dec si
    ret


; > Разбор единственного числового аргумента команды
; (Пропускает пробелы, читает число и требует пустой хвост)
; Параметры:
;  - si: указатель на аргументы команды
; Вывод:
;  - ax: число (uint16)
;  - CF (Carry Flag): 0 (Успех), 1 (Пусто / Не число / Мусор после числа)
parse_uint16_arg:
    ; Пропуск пробелов до аргумента
    call skip_spaces
    cmp byte [si], 0
    je .fail

    ; Парсинг числа
    call parse_uint16
    jc .fail

    ; После числа допустимы только пробелы
    push ax
    call skip_spaces
    cmp byte [si], 0
    pop ax
    jne .fail

.done:
    clc
    ret

.fail:
    stc
    ret


; > Пропуск пробелов с проверкой, что аргумент не пуст
; Параметры:
;  - si: указатель на аргументы команды
; Вывод:
;  - si: указатель на первый символ аргумента
;  - CF (Carry Flag): 0 (Аргумент есть), 1 (Аргумент пуст)
require_arg:
    call skip_spaces
    cmp byte [si], 0
    je .fail

.done:
    clc
    ret

.fail:
    stc
    ret