; ════════════════════════════════════════════════════════════════════════════
;  Realix · Vault — SHA-256
;  Потоковый хеш сообщения: init → update → final. Длина данных не ограничена.
;
;  Эталон    · tools/vault_forge.py (_pure_sha256)
;  Требует   · i386+ (32-битная арифметика в реальном режиме)
;  Регистры  · данные читаются через ds; контекст хранится в data-секции модуля
; ════════════════════════════════════════════════════════════════════════════

; Защита от повторного включения

%ifndef SHA256_ASM
%define SHA256_ASM


; > Инициализация контекста хеша (H = IV, сброс счётчиков)
sha256_init:
    pushad

    ; Копируем начальный вектор H0..H7
    mov si, sha_iv
    mov di, sha_h
    mov cx, 8
.copy_iv:
    mov eax, [si]
    mov [di], eax
    add si, 4
    add di, 4
    loop .copy_iv

    ; Сброс буфера и общей длины
    mov word [sha_fill], 0
    mov dword [sha_tot_lo], 0
    mov dword [sha_tot_hi], 0

    popad
    ret


; > Добавление данных в хеш
; Параметры:
;  - ds:si: указатель на данные
;  - cx: кол-во байт (0..65535)
sha256_update:
    pushad
    push es
    push ds
    pop es                  ; es = ds (для rep movsb)

    ; Обновление 64-битного счётчика длины (в байтах)
    movzx eax, cx
    add [sha_tot_lo], eax
    adc dword [sha_tot_hi], 0

.feed:
    test cx, cx
    jz .done

    ; take = min(64 - fill, cx)
    mov bx, [sha_fill]
    mov ax, 64
    sub ax, bx
    cmp ax, cx
    jbe .have_take
    mov ax, cx
.have_take:
    ; Копируем take байт в sha_block + fill
    mov di, sha_block
    add di, bx
    push cx
    mov cx, ax
    rep movsb               ; ds:si -> es:di, si и di продвигаются
    pop cx

    add [sha_fill], ax
    sub cx, ax

    ; Блок заполнен? — сжимаем
    cmp word [sha_fill], 64
    jne .feed
    call sha256_compress
    mov word [sha_fill], 0
    jmp .feed

.done:
    pop es
    popad
    ret


; > Завершение хеша (дополнение + вывод дайджеста)
; Параметры:
;  - ds:di: буфер на 32 байта под дайджест (big-endian)
sha256_final:
    pushad
    mov [sha_out], di       ; Сохраняем адрес вывода

    ; bits = total_bytes * 8 (64-битный сдвиг влево на 3)
    mov eax, [sha_tot_lo]
    mov edx, [sha_tot_hi]
    mov ebx, eax
    shl eax, 3
    shr ebx, 29
    shl edx, 3
    or edx, ebx
    mov [sha_bits_lo], eax
    mov [sha_bits_hi], edx

    ; Добавляем байт 0x80
    mov bx, [sha_fill]
    mov byte [sha_block + bx], 0x80
    inc bx

    ; Если не помещается длина (fill > 56) — дополняем и сжимаем блок
    cmp bx, 56
    jbe .pad_zeros
.fill_to_64:
    cmp bx, 64
    jae .extra_block
    mov byte [sha_block + bx], 0
    inc bx
    jmp .fill_to_64
.extra_block:
    call sha256_compress
    xor bx, bx

.pad_zeros:
    ; Нули до смещения 56
    cmp bx, 56
    jae .put_len
    mov byte [sha_block + bx], 0
    inc bx
    jmp .pad_zeros

.put_len:
    ; Длина в битах, big-endian: [56..59]=hi, [60..63]=lo
    mov eax, [sha_bits_hi]
    mov edx, eax
    shr edx, 24
    mov [sha_block + 56], dl
    mov edx, eax
    shr edx, 16
    mov [sha_block + 57], dl
    mov edx, eax
    shr edx, 8
    mov [sha_block + 58], dl
    mov [sha_block + 59], al

    mov eax, [sha_bits_lo]
    mov edx, eax
    shr edx, 24
    mov [sha_block + 60], dl
    mov edx, eax
    shr edx, 16
    mov [sha_block + 61], dl
    mov edx, eax
    shr edx, 8
    mov [sha_block + 62], dl
    mov [sha_block + 63], al

    call sha256_compress

    ; Вывод H0..H7 в буфер (big-endian)
    mov si, sha_h
    mov di, [sha_out]
    mov cx, 8
.write_out:
    mov eax, [si]
    mov edx, eax
    shr edx, 24
    mov [di + 0], dl
    mov edx, eax
    shr edx, 16
    mov [di + 1], dl
    mov edx, eax
    shr edx, 8
    mov [di + 2], dl
    mov [di + 3], al
    add si, 4
    add di, 4
    loop .write_out

    popad
    ret


; > Сжатие одного 64-байтного блока (sha_block) в состояние sha_h
sha256_compress:
    pushad

    ; --- Загрузка 16 слов (big-endian) в расписание W[0..15] ---
    mov si, sha_block
    mov di, sha_w
    mov cx, 16
.load_words:
    movzx eax, byte [si]
    shl eax, 8
    mov al, [si + 1]
    shl eax, 8
    mov al, [si + 2]
    shl eax, 8
    mov al, [si + 3]
    mov [di], eax
    add si, 4
    add di, 4
    loop .load_words

    ; --- Расширение W[16..63] ---
    mov cx, 48
    mov di, sha_w + 16 * 4
.extend:
    ; s0 = ror(W[i-15],7) ^ ror(W[i-15],18) ^ (W[i-15] >> 3)
    mov eax, [di - 15 * 4]
    mov ebx, eax
    ror eax, 7
    mov edx, ebx
    ror edx, 18
    xor eax, edx
    shr ebx, 3
    xor eax, ebx
    mov esi, eax            ; esi = s0
    ; s1 = ror(W[i-2],17) ^ ror(W[i-2],19) ^ (W[i-2] >> 10)
    mov eax, [di - 2 * 4]
    mov ebx, eax
    ror eax, 17
    mov edx, ebx
    ror edx, 19
    xor eax, edx
    shr ebx, 10
    xor eax, ebx            ; eax = s1
    add eax, esi            ; + s0
    add eax, [di - 16 * 4]  ; + W[i-16]
    add eax, [di - 7 * 4]   ; + W[i-7]
    mov [di], eax
    add di, 4
    loop .extend

    ; --- Рабочие переменные a..h = H0..H7 ---
    mov si, sha_h
    mov di, wv_a
    mov cx, 8
.copy_wv:
    mov eax, [si]
    mov [di], eax
    add si, 4
    add di, 4
    loop .copy_wv

    ; --- 64 раунда ---
    xor cx, cx              ; i = 0
.round:
    mov bx, cx
    shl bx, 2               ; bx = i * 4 (индекс в W и K)

    ; S1 = ror(e,6) ^ ror(e,11) ^ ror(e,25)
    mov eax, [wv_e]
    mov edx, eax
    ror eax, 6
    mov esi, edx
    ror esi, 11
    xor eax, esi
    mov esi, edx
    ror esi, 25
    xor eax, esi            ; eax = S1
    ; ch = (e & f) ^ (~e & g)
    mov esi, [wv_e]
    and esi, [wv_f]
    mov edi, [wv_e]
    not edi
    and edi, [wv_g]
    xor esi, edi            ; esi = ch
    add eax, esi
    add eax, [wv_h]
    add eax, [sha_k + bx]
    add eax, [sha_w + bx]
    mov [sha_t1], eax       ; t1 = h + S1 + ch + K[i] + W[i]

    ; S0 = ror(a,2) ^ ror(a,13) ^ ror(a,22)
    mov eax, [wv_a]
    mov edx, eax
    ror eax, 2
    mov esi, edx
    ror esi, 13
    xor eax, esi
    mov esi, edx
    ror esi, 22
    xor eax, esi            ; eax = S0
    ; maj = (a & b) ^ (a & c) ^ (b & c)
    mov esi, [wv_a]
    and esi, [wv_b]
    mov edi, [wv_a]
    and edi, [wv_c]
    xor esi, edi
    mov edi, [wv_b]
    and edi, [wv_c]
    xor esi, edi            ; esi = maj
    add eax, esi            ; eax = t2 = S0 + maj

    ; Ротация: h=g; g=f; f=e; e=d+t1; d=c; c=b; b=a; a=t1+t2
    mov edx, [wv_g]
    mov [wv_h], edx
    mov edx, [wv_f]
    mov [wv_g], edx
    mov edx, [wv_e]
    mov [wv_f], edx
    mov edx, [wv_d]
    add edx, [sha_t1]
    mov [wv_e], edx
    mov edx, [wv_c]
    mov [wv_d], edx
    mov edx, [wv_b]
    mov [wv_c], edx
    mov edx, [wv_a]
    mov [wv_b], edx
    mov edx, [sha_t1]
    add edx, eax            ; a = t1 + t2
    mov [wv_a], edx

    inc cx
    cmp cx, 64
    jb .round

    ; --- H += a..h ---
    mov si, wv_a
    mov di, sha_h
    mov cx, 8
.add_state:
    mov eax, [di]
    add eax, [si]
    mov [di], eax
    add si, 4
    add di, 4
    loop .add_state

    popad
    ret


; Начальный вектор (H0..H7)
sha_iv:
    dd 0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a
    dd 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19

; Раундовые константы (K0..K63)
sha_k:
    dd 0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5
    dd 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5
    dd 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3
    dd 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174
    dd 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc
    dd 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da
    dd 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7
    dd 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967
    dd 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13
    dd 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85
    dd 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3
    dd 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070
    dd 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5
    dd 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3
    dd 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208
    dd 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2

; Контекст хеша
sha_h:      times 8 dd 0    ; Текущее состояние H0..H7
sha_w:      times 64 dd 0   ; Расписание сообщения W[0..63]
wv_a:       dd 0            ; Рабочие переменные a..h (должны идти подряд)
wv_b:       dd 0
wv_c:       dd 0
wv_d:       dd 0
wv_e:       dd 0
wv_f:       dd 0
wv_g:       dd 0
wv_h:       dd 0
sha_block:  times 64 db 0   ; Текущий блок
sha_fill:   dw 0            ; Байт в блоке (0..63)
sha_tot_lo: dd 0            ; Общая длина (байты), младшие 32 бита
sha_tot_hi: dd 0            ; Общая длина (байты), старшие 32 бита
sha_bits_lo: dd 0           ; Длина в битах (младшие)
sha_bits_hi: dd 0           ; Длина в битах (старшие)
sha_t1:     dd 0            ; Временная t1
sha_out:    dw 0            ; Адрес вывода дайджеста

%endif
