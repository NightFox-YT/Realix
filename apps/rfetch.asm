; © Realix > realixfetch - System info tool (like neofetch/fastfetch)
; ø Copyright by @Ramix (лого - ASCII art, автор подписи "jgs", asciiart.eu)
; (07.09.26) v0.12
; ================
; ❗️ 32-битное RLX-приложение. Собирает данные о системе через новые
;    syscall'ы SYS_GET_UPTIME/SYS_GET_MEMINFO/SYS_GET_VERSION (int 0x80)
;    и CPU vendor string напрямую через CPUID (Не требует привилегий)

; Настройка компиляции
bits 32
org 0x300000

; Основные константы
%include 'shared/config.asm'

; Заголовок .RLX (16 байт, формат общий с 16-битными приложениями)
rlx_header:
    dw RLX_MAGIC               ; Магическое число для опознания
    dw RLX_MODE_32              ; Режим работы приложения
    dw app_entry - rlx_header  ; Оффсет точки входа
    dw 0                       ; Зарезервировано

app_entry:
    ; ❗️ exec.rs вызывает точку входа как `unsafe extern "C" fn()` - по соглашению C
    ; регистры EBX/ESI/EDI/EBP должны сохраняться вызываемой стороной. Приложение activно
    ; использует esi/ebx ниже, поэтому сохраняем их и восстанавливаем перед `ret`
    push ebx
    push esi
    push edi
    push ebp

    mov ah, SYS_CLEAR
    int 0x80

    ; Строка 1: лого + название
    mov esi, logo1
    call print_str
    mov esi, header_name
    call print_str
    call print_newline

    ; Строка 2: лого + разделитель
    mov esi, logo2
    call print_str
    mov esi, header_sep
    call print_str
    call print_newline

    ; Строка 3: лого + "OS: ..."
    mov esi, logo3
    call print_str
    mov esi, label_os
    call print_str
    call print_newline

    ; Строка 4: лого + "Kernel: ..." + версия (SYS_GET_VERSION)
    mov esi, logo4
    call print_str
    mov esi, label_kernel
    call print_str
    mov ah, SYS_GET_VERSION
    int 0x80              ; ecx - указатель на строку версии
    mov esi, ecx
    call print_str
    call print_newline

    ; Строка 5: лого + "Uptime: ..." (SYS_GET_UPTIME)
    mov esi, logo5
    call print_str
    mov esi, label_uptime
    call print_str
    mov ah, SYS_GET_UPTIME
    int 0x80              ; ecx - секунды с загрузки
    mov eax, ecx
    call print_dec32
    mov esi, str_seconds
    call print_str
    call print_newline

    ; Строка 6: лого + "Memory: ..." (SYS_GET_MEMINFO)
    mov esi, logo6
    call print_str
    mov esi, label_memory
    call print_str
    mov ah, SYS_GET_MEMINFO
    int 0x80              ; ecx - всего КБ, edx - свободно КБ
    push ecx               ; Сохраняем "всего" (print_dec32 портит ecx)
    mov eax, edx
    call print_dec32       ; Печатаем "свободно"
    mov esi, str_slash_kb
    call print_str
    pop eax                ; "Всего" обратно в eax
    call print_dec32       ; Печатаем "всего"
    mov esi, str_kb
    call print_str
    call print_newline

    ; Строка 7: лого + "CPU: ..." (CPUID vendor string)
    mov esi, logo7
    call print_str
    mov esi, label_cpu
    call print_str
    call print_cpu_vendor
    call print_newline

    call print_newline
    mov esi, msg_prompt
    call print_str

    mov ah, SYS_READ_KEY
    int 0x80

    mov ah, SYS_EXIT
    int 0x80

    pop ebp
    pop edi
    pop esi
    pop ebx
    ret

; > Печать строки по указателю (esi) до нуль-терминатора
print_str:
    push eax
    mov ah, SYS_PRINT_STRING
    int 0x80
    pop eax
    ret

; > Перевод строки (CRLF)
print_newline:
    push eax
    mov ah, SYS_PUTCHAR
    mov al, 0x0D
    int 0x80
    mov ah, SYS_PUTCHAR
    mov al, 0x0A
    int 0x80
    pop eax
    ret

; > Печать eax как десятичное число без знака
; ❗️ Портит eax, ebx, ecx, edx (Числовое значение - параметр в eax)
print_dec32:
    mov ebx, 10
    xor ecx, ecx            ; Счётчик цифр в стеке

    test eax, eax
    jnz .split_loop

    ; Особый случай: 0
    mov ah, SYS_PUTCHAR
    mov al, '0'
    int 0x80
    ret

.split_loop:
    ; Раскладываем число на цифры (с конца), складывая их в стек
    xor edx, edx
    div ebx                 ; eax / 10, остаток - в edx
    push edx
    inc ecx
    test eax, eax
    jnz .split_loop

.print_loop:
    ; Печатаем цифры в правильном порядке, снимая их со стека
    pop edx
    add dl, '0'
    mov ah, SYS_PUTCHAR
    mov al, dl
    int 0x80
    loop .print_loop
    ret

; > Печать 12-символьного CPU vendor string через CPUID(eax=0)
print_cpu_vendor:
    push eax
    push ebx
    push ecx
    push edx
    push esi

    xor eax, eax
    cpuid

    mov [cpu_vendor + 0], ebx
    mov [cpu_vendor + 4], edx
    mov [cpu_vendor + 8], ecx
    mov byte [cpu_vendor + 12], 0

    mov esi, cpu_vendor
    call print_str

    pop esi
    pop edx
    pop ecx
    pop ebx
    pop eax
    ret

data_start:

; Заголовок и подписи
header_name:  db 'Realix Nightly', 0
header_sep:   db '----------------', 0
label_os:     db 'OS: Realix Nightly (x86, Protected Mode)', 0
label_kernel: db 'Kernel: kernel32 (Rust, no_std) ', 0
label_uptime: db 'Uptime: ', 0
str_seconds:  db 's', 0
label_memory: db 'Memory: ', 0
str_slash_kb: db ' KB / ', 0
str_kb:       db ' KB', 0
label_cpu:    db 'CPU: ', 0
msg_prompt:   db 'Press any key to exit...', 0

; Лого (7 строк, выровнены до одинаковой ширины пробелами)
logo1: db "         _.._     ", 0
logo2: db "        .' .-'`   ", 0
logo3: db "       /  /       ", 0
logo4: db "       |  |       ", 0
logo5: db "       \  '.___.; ", 0
logo6: db "        '._  _.'  ", 0
logo7: db "           ``     ", 0

; Буфер для CPU vendor string (12 символов + нуль-терминатор)
cpu_vendor: times 13 db 0

data_end:

code_end:
