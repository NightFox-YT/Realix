; © Realix > FAT12
; (05.04.26) v0.06
; ================
; Зависимости: disk/read.asm, error_handler (функция)

; > Загрузка файла с диска
; Параметры:
;  - ds:si: имя файла (11 символов)
;  - ax: сегмент назначения файла
;  - bx: смещение назначения файла
;  - dl: номер диска
; Вывод:
;  - Успех: возвращает управление
;  - Ошибка: вызывает error_handler
load_file:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    ; Сохраняем входные данные
    mov [filename], si
    mov [file_segment], ax
    mov [file_offset], bx
    mov [drive_num], dl

    ; Проверка на наобходимость инициализации FAT параметров
    cmp byte [fat_initialized], 1
    je .read_root_dir

.fat_init:
    ; Чтение BPB параметров (С помощью es)
    push es
    xor ax, ax
    mov es, ax

    mov ax, [es:0x7C00 + 11]
    mov [bytes_per_sector], ax
    mov al, [es:0x7C00 + 13]
    mov [sectors_per_cluster], al
    mov ax, [es:0x7C00 + 14]
    mov [reserved_sectors], ax
    mov al, [es:0x7C00 + 16]
    mov [fat_count], al
    mov ax, [es:0x7C00 + 17]
    mov [dir_entries], ax
    mov ax, [es:0x7C00 + 22]
    mov [sectors_per_fat], ax
    mov ax, [es:0x7C00 + 24]
    mov [sectors_per_track], ax
    mov ax, [es:0x7C00 + 26]
    mov [heads], ax

    pop es

    ; Вычисление LBA корневого каталога
    ; > LBA = sectors_per_fat * fats + reserved
    mov ax, [sectors_per_fat]
    mov bl, [fat_count]
    xor bh, bh
    mul bx
    add ax, [reserved_sectors]
    mov [root_dir_lba], ax

    ; Вычисление размера корневого каталога
    ; > root_dir_size = (number_of_entries * 32) / bytes_per_sector
    mov ax, [dir_entries]
    shl ax, 5             ; *32 (ax = number_of_entries * 32)
    xor dx, dx
    div word [bytes_per_sector]

    ; Округление размера корневого каталога до целого вверх
    or dx, dx
    jz .save_root_size
    inc ax

.save_root_size:
    ; Обновление переменной fat_lba (LBA = root_lba + root_size)
    mov [root_dir_size], ax
    add ax, [root_dir_lba]
    mov [fat_lba], ax

    ; Обновляем флаг инициализации
    mov byte [fat_initialized], 1

; Чтение корневого каталога
.read_root_dir:
    ; Чтение корневого каталога
    mov ax, [root_dir_lba]
    mov cl, [root_dir_size]    ; Кол-во секторов - размер каталога
    mov dl, [drive_num]        ; Номер диска
    mov bx, 0x7E00             ; Адрес данных для записи
    push word [sectors_per_track]
    push word [heads]
    call disk_read

    ; Подготовка к поиску файла
    xor bx, bx      ; Кол-во пройденных записей корневого каталога
    mov di, 0x7E00  ; Адрес текущей записи корневого каталога

; Поиск файла
.search:
    ; Подготовка к сравнению названий
    mov si, [filename]
    mov cx, 11         ; Сравниваем названия до 11 символов

    ; Сравниваем по символу названия файлов, сохраняя адрес записи
    ; > si:di++ до cx == 0
    push di
    repe cmpsb
    pop di
    je .found

    ; Переход к следующей записи
    add di, 32              ; Увеличиваем адрес на размер записи (32 байта)
    inc bx                  ; Увеличиваем индекс записи
    cmp bx, [dir_entries]
    jl .search              ; Если не вышли за предел, продолжаем поиск

    ; Вышли за предел, => файла второго этапа загрузчика нет
    mov si, err_file_not_found
    jmp error_handler

.found:
    ; Обновление номера кластера (di - адрес записи корневого каталога)
    mov ax, [di + 26]       ; Поле первого кластера (Смещение 26 байтов)
    mov [file_cluster], ax  ; Обновляем переменную

    ; Чтение FAT таблицы
    mov ax, [reserved_sectors]
    mov cl, [sectors_per_fat]      ; Кол-во секторов - размер FAT
    mov dl, [drive_num]            ; Номер диска
    mov bx, 0x7E00                 ; Адрес данных для записи
    push word [sectors_per_track]
    push word [heads]
    call disk_read

    ; Установка сегмента и смещения для чтения файла
    mov bx, [file_segment]
    mov es, bx
    mov bx, [file_offset]

; Чтение файла и обработка FAT цепочки
.load_loop:
    ; Чтение следующего кластера (dl содержит номер диска)
    push word [sectors_per_track]
    push word [heads]
    xor ch, ch
    mov cl, [sectors_per_cluster]  ; Кол-во секторов
    mov dl, [drive_num]            ; Номер диска

    ; Вычисление LBA кластера
    ; > LBA = (file_cluster - 2) * sectors_per_cluster + fat_lba
    mov ax, [file_cluster]
    sub ax, 2
    mul cx
    add ax, [fat_lba]
    call disk_read

    ; Увеличиваем адрес смещения назначения на кол-во прочитанных байт
    xor ah, ah
    mov al, [sectors_per_cluster]
    mul word [bytes_per_sector]
    add bx, ax
    jnc .load_loop_continue

    ; Сдвигаем es на след. параграф (+64 КБ)
    mov ax, es
    add ax, 0x1000
    mov es, ax

; Продолжение чтения файла
.load_loop_continue:
    ; Вычисляем LBA следующего кластера
    ; > ax - индекс записи, dx - Cluster % 2
    mov ax, [file_cluster]
    mov cx, 3
    mul cx
    mov cx, 2
    div cx

    ; Считывание записи из таблицы FAT по индексу (ax)
    mov si, 0x7E00
    add si, ax
    mov ax, [ds:si]

    ; Проверка чётности кластера
    or dx, dx
    jz .even_cluster

; Нечётный кластер (Оставляем старшие 12 бит)
.odd_cluster:
    shr ax, 4
    jmp .next_cluster

; Чётный кластер (Оставляем младшие 12 бит)
.even_cluster:
    and ax, 0x0FFF

; Обработка следующего кластера
.next_cluster: 
    ; Проверка на конец файла
    cmp ax, 0x0FF8
    jae .done

    ; Обновляем номер текущего кластера, продолжая чтение
    mov [file_cluster], ax
    jmp .load_loop

.done:
    pop es
    pop di
    pop si
    pop dx
    pop cx
    pop bx
    pop ax
    
    ret


; Параметры файла
filename:     dw 0
file_segment: dw 0
file_offset:  dw 0
file_cluster: dw 0

; BPB параметры FAT
bytes_per_sector:    dw 0
sectors_per_cluster: db 0
reserved_sectors:    dw 0
fat_count:           db 0
dir_entries:         dw 0
sectors_per_fat:     dw 0
sectors_per_track:   dw 0
heads:               dw 0
drive_num:           db 0

; Init параметры FAT
root_dir_lba:    dw 0
root_dir_size:   dw 0
fat_lba:         dw 0
fat_initialized: db 0

; Ошибки
err_fat_init:       db '[!] FAT initialization failed!', 0
err_file_not_found: db '[!] File not found!', 0