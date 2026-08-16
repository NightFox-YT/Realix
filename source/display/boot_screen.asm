; © Realix > Display: Boot Screen
; (15.08.26) v0.12
; ================
; ❗️ Зависимости: bios-api/io/*, display/memory

; > Вывод загрузочного экрана (Initrix | Текстовый режим)
; Параметры:
;  - ax: кол-во "нижней" памяти
;  - ebx: кол-во всей памяти
;  - cx: кол-во записей в карте памяти
show_boot_screen:
    push eax
    push ebx
    push si

    ; Вывод заголовка
    call clear_screen
    mov si, str_title
    call print
    call print_beep_char

.show_memory_info:
    ; Вывод диагностической информации о памяти
    ; (Некоторые аргументы находятся уже в нужных регистрах)
    call show_lower_memory
    call print_new_line

    mov eax, ebx
    call show_usable_memory
    call print_new_line

    call show_map_entries_cnt
    call print_new_line
    call print_new_line

.done:
    pop si
    pop ebx
    pop eax
    ret


; Строки
str_title:
    db '     Realix ', OS_VERSION, ENTER
    db '(C) NightFox developer', ENTER, ENTER, 0