; Realix > Read file
; (C) v0.04 | 25.08.25
; ==================

; [Disk] Чтение файла с диска
; Параметры:
;  - Переменная file_name
;  - Константы FILE_LOAD_SEGMENT и FILE_LOAD_OFFSET
read_file:
    ; Подготовка к чтению корневого каталога > Вычисление LBA корневого каталога (sectors_per_fat * fats + reserved)
    mov ax, [bpb_sectors_per_fat]  ; ax = sectors_per_fat
    xor bx, bx
    mov bl, [bpb_fats]             ; bl = fats
    mul bx                         ; Умножаем на bx: ax = (sectors_per_fat * fats)
    add ax, [bpb_reserved_sectors] ; ax = LBA корневого каталога

    ; *Сохраняем LBA корневого каталога
    push ax
    mov cx, ax

    ; Вычисление размера корневого каталога ((32 * number_of_entries) / bytes_per_sector)
    mov ax, [bpb_dir_entries]       ; ax = number_of_entries
    shl ax, 5                       ; ax = number_of_entries * 32
    xor dx, dx
    div word [bpb_bytes_per_sector] ; ax = (32 * number_of_entries) / bytes_per_sector

    or dx, dx    ; Проверка dx (Остатка) на 0
    jz .read_root_dir
    inc ax       ; dx != 0, округлим, добавив 1 (Есть частично заполненный сектор)

; Чтение корневого каталога
.read_root_dir:
    ; Обновление переменной root_dir_end
    add cx, ax
    mov [root_dir_end], cx

    ; Чтение корневого каталога
    pop ax                     ; *Восстанавливаем LBA корневого каталога
    mov cl, al                 ; Кол-во секторов для чтения (Размер корневого каталога)
    mov dl, [ebr_drive_number] ; Номер диска
    mov bx, 0x7E00             ; Адрес данных для записи
    call disk_read

    ; Подготовка к поиску файла
    xor bx, bx
    mov di, 0x7E00 ; Перемещаем адрес первой записи корневого каталога в di

; Поиск файла
.search_file:
    mov si, file_name
    mov cx, 11 ; Сравниваем названия файлов до 11 символов
    push di
    repe cmpsb ; Сравниваем по символу название файлов
    pop di
    je .found_file

    add di, 32                ; Переходим к следующей записи (Увеличиваем адрес на 32)
    inc bx                    ; Увеличиваем индекс записи
    cmp bx, [bpb_dir_entries]
    jl .search_file         ; Если не вышли за предел, продолжаем поиск

    jmp file_not_found_error ; Вышли за предел, => файла ядра нет...

; Найден file
.found_file:
    ; di должен содержать адрес нужной записи корневого каталога
    mov ax, [di + 26]         ; Поле первого кластера (Смещение 26 байтов)
    mov [file_cluster], ax ; Обновляем переменную

    ; Загрузка FAT таблицы с диска в память
    mov ax, [bpb_reserved_sectors] ; LBA
    mov cl, [bpb_sectors_per_fat]  ; Кол-во секторов для чтения
    mov bx, 0x7E00                 ; Адрес данных для записи
    mov dl, [ebr_drive_number]     ; Номер диска
    call disk_read

    ; Обновление сегментов и смещения для file
    mov bx, FILE_LOAD_SEGMENT
    mov es, bx
    mov bx, FILE_LOAD_OFFSET

; Чтение file и обработка FAT цепочки
.load_file_loop:
    ; Вычисление LBA кластера
    mov ax, [file_cluster] ; Сохраняем в ax номер текущего кластера
    sub ax, 2                ; ax = file_cluster - 2
    mul cx                   ; ax = (file_cluster - 2) * sectors_per_cluster
    add ax, [root_dir_end]   ; (Кластер) ax = (file_cluster - 2) * sectors_per_cluster + root_dir_end

    ; Чтение кластера
    mov cl, [bpb_sectors_per_cluster] ; Кол-во секторов для чтения
    mov dl, [ebr_drive_number]        ; Номер диска
    call disk_read

    ; Увеличиваем адрес смещения file на количество прочитанных кластеров, так как в текущий мы записали
    push ax                           ; *Сохраняем LBA кластера
    mov ax, [bpb_sectors_per_cluster]
    mov bx, [bpb_bytes_per_sector]
    mul bx
    mov cx, ax                        
    pop ax                            ; *Восстанавливаем LBA клаcтера
    add bx, cx                        ; bx += sectors_per_cluster * bytes_per_sector

    ; Вычисляем LBA следующего кластера
    mov ax, [file_cluster]
    mov cx, 3
    mul cx
    mov cx, 2
    div cx           ; ax = индекс записи, dx = (остаток) кластер / 2

    mov si, 0x7E00
    add si, ax
    mov ax, [ds:si]  ; Считывание записи из таблицы FAT по индексу (ax)

    or dx, dx        ; Проверяем остаток от деления
    jz .even_cluster ; Если чётный кластер, переходим .even_cluster

; Нечётный кластер
.odd_cluster:
    shr ax, 4
    jmp .next_cluster_after

; Чётный кластер
.even_cluster:
    and ax, 0x0FFF

; ОЛбработка следующего кластера
.next_cluster_after:
    cmp ax, 0x0FF8         ; Проверяем на конец файла
    jae .read_file_finish  ; Если да, переходим к .read_file_finish

    mov [file_cluster], ax ; Обновляем номер кластера
    jmp .load_file_loop

; Заканчиваем чтение файла
.read_file_finish: 
    ; Настройка сегментов и регистров для file
    mov ax, FILE_LOAD_SEGMENT
    mov ds, ax
    mov es, ax
    mov dl, [ebr_drive_number]

    ; Переходим к file...
    jmp FILE_LOAD_SEGMENT:FILE_LOAD_OFFSET

; [Disk] Ошибки
file_not_found_error:
    mov si, err_initrix_not_found
    call print
    jmp error_handler

file_cluster: dw 0
root_dir_end: dw 0