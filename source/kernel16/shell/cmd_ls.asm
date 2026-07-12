; © Realix > LS Command
; (12.07.26) v0.09
; ================
; ❗️ Зависимости: bios-api/disk/read.asm, bios-api/fat12/init.asm

; > Команда вывода списка файлов в корневом каталоге
cmd_ls:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    ; Читаем корневой каталог в память (используем буфер выше загрузчика)
    xor ax, ax
    mov es, ax
    mov ax, [root_dir_lba]
    mov cx, [root_dir_size]
    mov dl, [drive_num]
    mov bx, 0x0600          ; Используем буфер 0x0600 (выше FAT)
    call disk_read

    ; Подготовка к выводу
    xor bx, bx              ; Счётчик записей
    mov di, 0x0600          ; Адрес текущей записи
    mov cx, [dir_entries]   ; Максимум записей

    mov si, str_ls_header
    call print
    call print_new_line

.loop:
    ; Проверка на конец каталога
    cmp bx, cx
    jae .done

    ; Проверка, что запись не пустая (первый байт 0x00 или 0xE5)
    mov al, [es:di]
    cmp al, 0x00
    je .done                ; Конец каталога
    cmp al, 0xE5
    je .next_entry          ; Удалённый файл

    ; Проверка атрибутов (Volume Label, Directory, Long File Name)
    mov al, [es:di + 11]
    test al, 0x08           ; Volume Label
    jnz .next_entry
    test al, 0x10           ; Directory
    jnz .next_entry
    test al, 0x0F           ; Long File Name
    jnz .next_entry

    ; Вывод имени файла (8+3 символа)
    push cx
    mov cx, 8
    mov si, di
    
.print_name:
    mov al, [es:si]
    cmp al, ' '
    je .check_ext
    call print_char
    inc si
    loop .print_name
    jmp .print_ext

.check_ext:
    ; Пропускаем пробелы в имени
    inc si
    loop .print_name

.print_ext:
    ; Если есть расширение
    mov al, [es:di + 8]
    cmp al, ' '
    je .end_print
    
    mov al, '.'
    call print_char
    
    mov cx, 3
    mov si, di
    add si, 8
    
.print_ext_loop:
    mov al, [es:si]
    cmp al, ' '
    je .end_print
    call print_char
    inc si
    loop .print_ext_loop

.end_print:
    ; Вывод размера файла
    push di
    mov si, str_tab
    call print
    
    mov ax, [es:di + 28]    ; Младшие 16 бит размера
    mov dx, [es:di + 30]    ; Старшие 16 бит размера
    
    ; Если размер > 65535, выводим как ">64K"
    test dx, dx
    jnz .large_file
    
    call print_dec16
    mov si, str_bytes
    call print
    jmp .next

.large_file:
    mov si, str_large
    call print

.next:
    call print_new_line
    pop di

.next_entry:
    ; Переход к следующей записи
    add di, 32
    inc bx
    jmp .loop

.done:
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    ret

; Строки
str_ls_header: db 'Files in root directory:', 0
str_tab:       db '  -  ', 0
str_bytes:     db ' bytes', 0
str_large:     db '>64K', 0
