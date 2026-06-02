; © Realix > FAT12: File Open
; (02.06.26) v0.05
; ================
; Зависимости: fat12/init.asm, disk/read.asm, error_handler (функция)

; > Загрузка файла с диска в память
; Параметры:
;  - ds:si: имя файла (11 символов)
;  - cx: сегмент назначения файла
;  - bx: смещение назначения файла
;  - dl: номер диска
; Вывод:
;  - Успех: возвращает управление
;  - Ошибка: вызывает error_handler
file_open:
    push ax
    push bx
    push cx
    push dx
    push si
    push di
    push es

    ; Сохраняем входные данные
    mov [filename], si
    mov [file_segment], cx
    mov [file_offset], bx
    mov [drive_num], dl

    ; Перевод адреса загрузки файла в линейный.
    ; ax - Нижние биты линейного адреса (16 бит)
    ; dx - Старшие биты линейного адреса (4 бита)
    mov ax, cx
    shl ax, 4
    mov dx, cx
    shr dx, 12
    add ax, bx
    adc dx, 0

    ; Проверка, что адрес записи не затирает буфер.
    cmp dx, 0x0
    mov si, err_buffer_overlap
    je error_handler

    ; Вызываем инициализацию FAT параметров
    call fat_init

; Чтение корневого каталога
.read_root_dir:
    ; Читаем Root directory в память
    xor ax, ax
    mov es, ax
    mov ax, [root_dir_lba]
    mov cx, [root_dir_size]
    mov dl, [drive_num]
    mov bx, 0x0500
    push word [sectors_per_track]
    push word [heads]
    call disk_read

    ; Подготовка к поиску файла
    xor bx, bx      ; Кол-во пройденных записей корневого каталога
    mov di, 0x0500  ; Адрес текущей записи корневого каталога

; Поиск файла
.search:
    ; Подготовка к сравнению названий (до 11 символов)
    mov si, [filename]
    mov cx, 11

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
    mov ax, [es:di + 26]    ; Поле первого кластера (Смещение 26 байтов)
    mov [file_cluster], ax  ; Обновляем переменную

    ; Читаем FAT в память
    xor ax, ax
    mov es, ax
    mov ax, [reserved_sectors]
    mov cx, [sectors_per_fat]
    mov dl, [drive_num]
    mov bx, 0x0500
    push word [sectors_per_track]
    push word [heads]
    call disk_read

    ; Установка сегмента и смещения для чтения файла
    mov bx, [file_segment]
    mov es, bx
    mov bx, [file_offset]

; Чтение файла и обработка FAT цепочки
.load_loop:
    ; Вычисление LBA кластера
    ; > LBA = (file_cluster - 2) * sectors_per_cluster + data_lba
    xor ch, ch
    mov cl, [sectors_per_cluster]
    mov ax, [file_cluster]
    sub ax, 2
    mul cx
    add ax, [data_lba]

    ; Чтение следующего кластера
    mov dl, [drive_num]
    push word [sectors_per_track]
    push word [heads]
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
    xor bx, bx

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
    push es
    xor cx, cx
    mov es, cx

    mov si, 0x0500
    add si, ax
    mov ax, [es:si]

    pop es

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

; Подключение FAT12: Init модуля
%include "fat12/init.asm"

; Параметры файла
filename:     dw 0
file_segment: dw 0
file_offset:  dw 0
file_cluster: dw 0

; Параметры устройства загрузки
drive_num: db 0

; Ошибки
err_file_not_found: db '[!] File not found!', 0
err_buffer_overlap: db '[!] The destination file address will overwrite the FAT buffer!', 0