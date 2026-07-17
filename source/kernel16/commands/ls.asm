; © Realix > LS Command
; (17.07.26) v0.1
; ================
; ❗️ Зависимости: bios-api/disk/read.asm, bios-api/fat12/init.asm,
;                 kernel16/io: print & print_ctrl

; ! Требуется инициализация FAT12 через `fat12_init`
%include "bios-api/fat12/init.asm"

; Основные константы
%include 'shared/config.asm'

; > Команда вывода списка файлов в корневом каталоге
cmd_ls:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    ; Читаем корневой каталог в память
    xor ax, ax
    mov es, ax
    mov ax, [root_dir_lba]
    mov cx, [root_dir_size]
    mov dl, [drive_num]
    mov bx, FAT_BUFFER_ADDR
    call disk_read

    ; Подготовка к выводу информации о файлах
    xor bx, bx               ; Счётчик пройденных записей
    mov di, FAT_BUFFER_ADDR  ; Смещение текущей записи

    mov si, str_ls_header
    call print
    call print_new_line

.loop:
    ; Проверка на конец каталога
    cmp bx, [dir_entries]
    jae .done

    ; Проверка: Запись не пустая (первый байт 0x00 или 0xE5)
    mov al, [es:di]
    cmp al, 0x00
    je .done        ; Конец каталога
    cmp al, 0xE5
    je .next_entry  ; Удалённый файл

    ; Проверка атрибутов (Volume Label, Directory, Long File Name)
    mov al, [es:di + 11]
    test al, 0x08    ; Volume Label
    jnz .next_entry
    test al, 0x10    ; Directory
    jnz .next_entry
    test al, 0x0F    ; Long File Name
    jnz .next_entry

    ; Подготовка к выводу имени файла (формат 8.3)
    mov cx, 8
    mov si, di
    
.print_name:
    ; Читаем имя файла и выводим
    mov al, [es:si]
    cmp al, ' '
    je .check_ext    ; Пробел -> Имя файла уже закончилось
    call print_char

    ; Переходим к след. символу, если считали 8 > к расширению
    inc si
    loop .print_name
    jmp .print_ext

.check_ext:
    ; Пропускаем оставшиеся пробелы в имени
    inc si
    loop .print_name

.print_ext:
    ; Читаем первый символ расширения файла
    mov al, [es:di + 8]
    cmp al, ' '
    je .print_size  ; Пробел -> Расширения нет
    
    ; Печатаем точку-разделитель
    mov al, '.'
    call print_char
    
    ; Подготовка к выводу расширения файла
    mov cx, 3
    mov si, di
    add si, 8
    
.print_ext_loop:
    ; Читаем расширение файла и выводим
    mov al, [es:si]
    cmp al, ' '
    je .print_size
    call print_char

    ; Переходим к след. символу, если считали 3 > идём дальше
    inc si
    loop .print_ext_loop

.print_size:
    ; Вывод размера файла
    push di
    mov si, str_tab
    call print
    
    ; Считывание 32-битного размера
    mov ax, [es:di + 28]  ; Младшие 16 бит размера
    mov dx, [es:di + 30]  ; Старшие 16 бит размера
    
    ; Если размер > 65535, выводим как ">64K"
    test dx, dx
    jnz .large_file
    
    ; Выводим младшие 16 бит размера с префиксом
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
str_tab:       db ' - ', 0
str_bytes:     db ' bytes', 0
str_large:     db '>64 Kilobytes', 0
