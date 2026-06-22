; ════════════════════════════════════════════════════════════════════════════
;  Realix · Vault — открытие хранилища по паролю
;  Доступ дают не сравнением пароля, а расшифровкой: пароль и есть ключ.
;  Нет пароля — на диске только шум, и разблокировать его нечем.
;
;  Криптосхема (зеркало tools/vault_forge.py):
;    master       = PBKDF2-HMAC-SHA256(password, salt, iters, 64)
;    enc_key      = master[0:32]      mac_key = master[32:64]
;    keystream(i) = HMAC-SHA256(enc_key, nonce || BE32(i))
;    plain        = cipher XOR keystream
;    tag          = HMAC-SHA256(mac_key, nonce || cipher)   (Encrypt-then-MAC)
;
;  Зависит · security/pbkdf2.asm, bios-api/fat12/file_open.asm,
;            kernel16/io/print.asm, kernel16/io/print_nl.asm, error_handler
;
;  Codded by Nixort <3
; ════════════════════════════════════════════════════════════════════════════

; Защита от повторного включения

%ifndef VAULT_ASM
%define VAULT_ASM

%include 'config.asm'
%include 'security/pbkdf2.asm'

; Смещения полей в VAULT.BIN (формат из vault_forge.py)
VLT_ITERS    equ 8          ; dd: число итераций PBKDF2
VLT_SALT     equ 12         ; 16 байт
VLT_NONCE    equ 28         ; 12 байт
VLT_PLEN     equ 40         ; dd: длина открытого текста
VLT_TAG      equ 44         ; 32 байт
VLT_CIPHER   equ 76         ; шифртекст
VLT_SALT_LEN  equ 16
VLT_NONCE_LEN equ 12


; > Инициализация хранилища: загрузка VAULT.BIN и разбор заголовка
; (Читает номер диска из PCINFO, инициализирует диск/FAT12, грузит файл)
; Вывод:
;  - Успех: возвращает управление
;  - Ошибка (нет файла / плохой магик / слишком большой): error_handler
vault_init:
    pushad

    ; Номер загрузочного диска из структуры PCINFO (сегмент 0)
    push es
    xor ax, ax
    mov es, ax
    mov al, [es:PCINFO_ADDR + 2]
    mov [vault_drive], al
    pop es

    ; Инициализация драйверов диска и FAT12
    mov dl, [vault_drive]
    call disk_init
    call fat12_init

    ; Загрузка VAULT.BIN в буфер vault_blob (текущий сегмент ядра)
    mov si, vault_filename
    mov cx, KERNEL_LOAD_SEGMENT
    mov bx, vault_blob
    mov dl, [vault_drive]
    call file_open

    ; Проверка магической подписи 'RVLT'
    mov si, vault_blob
    mov di, vault_magic
    mov cx, 4
    repe cmpsb
    jne .bad_magic

    ; Копируем число итераций и длину полезного груза
    mov eax, [vault_blob + VLT_ITERS]
    mov [pbk_iters], eax

    ; Длина груза (берём 16 бит; старшие должны быть нулевыми)
    mov eax, [vault_blob + VLT_PLEN]
    cmp eax, VAULT_PLAIN_MAX
    ja .too_big
    mov [vault_plen], ax

    ; Параметры соли для PBKDF2
    mov word [pbk_salt_ptr], vault_blob + VLT_SALT
    mov word [pbk_salt_len], VLT_SALT_LEN

    popad
    ret

.bad_magic:
    mov si, err_vault_magic
    jmp error_handler
.too_big:
    mov si, err_vault_big
    jmp error_handler


; > Интерактивное открытие хранилища (запрос пароля в цикле)
; Вывод:
;  - Успех: полезный груз расшифрован в vault_plain, возврат с CF = 0
;  - (Неверный пароль не завершает функцию — повторный запрос)
vault_unlock:
    push ax
    push si
    push di

.attempt:
    call vault_prompt       ; -> pw_buffer, pw_len

    ; master = PBKDF2(pw, salt, iters)
    mov word [pbk_pw_ptr], pw_buffer
    mov ax, [pw_len]
    mov [pbk_pw_len], ax
    mov word [pbk_out], vault_keys

    mov si, msg_deriving
    call print
    call pbkdf2_sha256

    ; Проверка целостности: tag = HMAC(mac_key, nonce || cipher)
    call vault_mac          ; -> vault_tag

    ; Сравнение с хранимым тегом за постоянное время (без раннего выхода)
    mov si, vault_tag
    mov di, vault_blob + VLT_TAG
    xor ah, ah              ; аккумулятор различий
    mov cx, 32
.cmp_loop:
    mov al, [si]
    xor al, [di]
    or ah, al
    inc si
    inc di
    loop .cmp_loop
    test ah, ah
    jnz .wrong

    ; Пароль верен — расшифровываем груз
    call vault_decrypt
    mov si, msg_unlocked
    call print

    clc
    pop di
    pop si
    pop ax
    ret

.wrong:
    mov si, err_vault_wrong
    call print
    jmp .attempt


; > Получение расшифрованного груза (публичный API для потребителей)
; Вывод:
;  - ds:si: указатель на открытый текст
;  - cx: длина груза
vault_get_secret:
    mov si, vault_plain
    mov cx, [vault_plen]
    ret


; > Запрос пароля с маскированием ввода (эхо «*»)
; Вывод:
;  - pw_buffer: введённый пароль (без нуль-терминатора для крипто)
;  - pw_len: длина пароля
vault_prompt:
    push ax
    push bx
    push di

    mov si, msg_password
    call print

    xor bx, bx
    mov di, pw_buffer

.key_loop:
    xor ah, ah
    int 0x16

    ; ENTER — завершаем
    cmp al, ENTER_KEY
    je .enter

    ; Backspace — стираем последний символ
    cmp al, BACKSPACE_KEY
    je .backspace

    ; Переполнение буфера пароля
    cmp bx, INPUT_BUFFER_LEN
    jae .key_loop

    ; Игнор управляющих символов
    cmp al, 0x20
    jb .key_loop

    ; Эхо «*» вместо символа
    mov [di], al
    inc di
    inc bx

    push bx
    mov ah, 0x0E
    xor bx, bx
    mov al, '*'
    int 0x10
    pop bx
    jmp .key_loop

.backspace:
    test bx, bx
    jz .key_loop
    dec di
    dec bx
    mov byte [di], 0

    push bx
    mov ah, 0x0E
    xor bx, bx
    mov al, 0x08
    int 0x10
    mov al, ' '
    int 0x10
    mov al, 0x08
    int 0x10
    pop bx
    jmp .key_loop

.enter:
    mov [pw_len], bx
    call print_new_line

    pop di
    pop bx
    pop ax
    ret


; > Вычисление тега: HMAC-SHA256(mac_key, nonce || cipher) -> vault_tag
; (Специализированный HMAC: сообщение склеивается из двух областей потоком)
vault_mac:
    pushad
    push es
    push ds
    pop es

    ; K0 = mac_key (32 байта), дополнено нулями до 64
    mov di, hmac_k0
    mov cx, 64
    xor al, al
    rep stosb
    mov si, vault_keys + 32
    mov di, hmac_k0
    mov cx, 32
    rep movsb

    ; Внутренний: SHA(ipad || nonce || cipher)
    mov si, hmac_k0
    mov di, hmac_pad
    mov cx, 64
.ipad:
    mov al, [si]
    xor al, 0x36
    mov [di], al
    inc si
    inc di
    loop .ipad

    call sha256_init
    mov si, hmac_pad
    mov cx, 64
    call sha256_update
    mov si, vault_blob + VLT_NONCE
    mov cx, VLT_NONCE_LEN
    call sha256_update
    mov si, vault_blob + VLT_CIPHER
    mov cx, [vault_plen]
    call sha256_update
    mov di, hmac_inner
    call sha256_final

    ; Внешний: SHA(opad || inner)
    mov si, hmac_k0
    mov di, hmac_pad
    mov cx, 64
.opad:
    mov al, [si]
    xor al, 0x5c
    mov [di], al
    inc si
    inc di
    loop .opad

    call sha256_init
    mov si, hmac_pad
    mov cx, 64
    call sha256_update
    mov si, hmac_inner
    mov cx, 32
    call sha256_update
    mov di, vault_tag
    call sha256_final

    pop es
    popad
    ret


; > Расшифровка груза: vault_plain = cipher XOR keystream
; (Ключевой поток — HMAC(enc_key, nonce || BE32(счётчик)) по блокам по 32 байта)
vault_decrypt:
    pushad
    push es
    push ds
    pop es

    mov word [vault_ctr], 0
    xor bx, bx              ; bx = смещение в грузе
.block:
    cmp bx, [vault_plen]
    jae .done

    ; Собираем вход PRF: nonce || BE32(ctr)
    mov si, vault_blob + VLT_NONCE
    mov di, vault_ksin
    mov cx, VLT_NONCE_LEN
    rep movsb
    mov byte [di + 0], 0
    mov byte [di + 1], 0
    mov ax, [vault_ctr]
    mov [di + 2], ah
    mov [di + 3], al

    ; Блок ключевого потока
    mov word [hmac_key_ptr], vault_keys
    mov word [hmac_key_len], 32
    mov word [hmac_msg_ptr], vault_ksin
    mov word [hmac_msg_len], 16
    mov word [hmac_out], vault_ks
    call hmac_sha256

    ; n = min(32, plen - bx)
    mov cx, [vault_plen]
    sub cx, bx
    cmp cx, 32
    jbe .have_n
    mov cx, 32
.have_n:
    ; plain[bx..] = cipher[bx..] XOR ks[0..n)
    mov si, vault_blob + VLT_CIPHER
    add si, bx
    mov di, vault_plain
    add di, bx
    mov bp, vault_ks
.xor_bytes:
    mov al, [si]
    xor al, [bp]
    mov [di], al
    inc si
    inc di
    inc bp
    dec cx
    jnz .xor_bytes

    add bx, 32
    inc word [vault_ctr]
    jmp .block

.done:
    pop es
    popad
    ret


; Сообщения и строки
msg_password:    db ENTER, 'Vault password: ', 0
msg_deriving:    db '[*] Deriving key...', ENTER, 0
msg_unlocked:    db '[+] Vault unlocked.', ENTER, 0
err_vault_wrong: db '[!] Wrong password.', ENTER, 0
err_vault_magic: db '[!] Bad vault signature!', 0
err_vault_big:   db '[!] Vault payload too large!', 0

; Постоянные
vault_filename: db 'VAULT   BIN'   ; Имя файла FAT12 (11 символов)
vault_magic:    db 'RVLT'

; Данные хранилища
vault_drive: db 0
vault_plen:  dw 0            ; Длина открытого текста
vault_ctr:   dw 0            ; Счётчик блоков ключевого потока
vault_keys:  times 64 db 0   ; enc_key(32) || mac_key(32)
vault_ks:    times 32 db 0   ; Текущий блок ключевого потока
vault_ksin:  times 16 db 0   ; nonce || BE32(ctr)
vault_tag:   times 32 db 0   ; Вычисленный тег
vault_blob:  times VAULT_BLOB_MAX  db 0   ; Загруженный VAULT.BIN
vault_plain: times VAULT_PLAIN_MAX db 0   ; Расшифрованный груз
pw_buffer:   times 65 db 0   ; Буфер пароля
pw_len:      dw 0

%endif
