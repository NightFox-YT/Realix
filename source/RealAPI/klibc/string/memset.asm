;; memset.asm, the realix libc (klibc and rlibc)
;; WARNING: get from musl libc.
;; Copyright (c) 2015-2026 Musl Developers
;; Copyright (c) 2026-present Alexander Silaev

global MemSet
section .text

MemSet:
    ; Входные параметры из стека (cdecl):
    ; [esp + 4]  -> dst (указатель)
    ; [esp + 8]  -> c   (байт заполнения)
    ; [esp + 12] -> n   (количество байт)

    mov ecx, [esp + 12]     ; n
    cmp ecx, 62
    ja .label2              ; если n > 62, уходим на блочное заполнение

    mov dl, [esp + 8]       ; c
    mov eax, [esp + 4]      ; dst
    test ecx, ecx
    jz .label1              ; если n == 0, выходим

    mov dh, dl

    mov [eax], dl
    mov [eax + ecx - 1], dl
    cmp ecx, 2
    jbe .label1

    mov [eax + 1], dx
    mov [eax + ecx - 3], dx ; -1-2 = -3
    cmp ecx, 6
    jbe .label1

    shl edx, 16
    mov dl, [esp + 8]
    mov dh, [esp + 8]

    mov [eax + 3], edx      ; 1+2 = 3
    mov [eax + ecx - 7], edx ; -1-2-4 = -7
    cmp ecx, 14
    jbe .label1

    mov [eax + 7], edx      ; 1+2+4 = 7
    mov [eax + 11], edx     ; 1+2+4+4 = 11
    mov [eax + ecx - 15], edx ; -1-2-4-8 = -15
    mov [eax + ecx - 11], edx ; -1-2-4-4 = -11
    cmp ecx, 30
    jbe .label1

    mov [eax + 15], edx     ; 1+2+4+8 = 15
    mov [eax + 19], edx     ; 1+2+4+8+4 = 19
    mov [eax + 23], edx     ; 1+2+4+8+8 = 23
    mov [eax + 27], edx     ; 1+2+4+8+12 = 27
    mov [eax + ecx - 31], edx ; -1-2-4-8-16 = -31
    mov [eax + ecx - 27], edx ; -1-2-4-8-12 = -27
    mov [eax + ecx - 23], edx ; -1-2-4-8-8 = -23
    mov [eax + ecx - 19], edx ; -1-2-4-8-4 = -19

.label1:
    ret                     ; Возвращаем eax (dst)

.label2:
    push edi                ; Сохраняем EDI в стек (теперь смещения аргументов увеличились на 4!)
    movzx eax, byte [esp + 12] ; Загружаем байт 'c' (был +8, стал +12 из-за push)
    imul eax, 0x01010101    ; азмножаем байт на все 4 байта регистра EAX
    mov edi, [esp + 8]      ; EDI = dst (был +4, стал +8)
    test edi, 15            ; Проверяем выравнивание адреса по 16 байт
    mov [edi + ecx - 4], eax ; Записываем последние 4 байта в самый конец буфера заранее
    jnz .label2_unaligned   ; Если не выровнено по 16 байт, идем выравнивать

.label1_loop:
    push edi                ; Сохраняем исходный EDI
    mov eax, [esp + 8]      ; Получаем значение для заполнения
    
    mov edi, [esp + 4]      ; Загружаем dst в EDI
    mov ecx, [esp + 12]     ; Загружаем n-ое количество байт
    shr ecx, 2              ; Делим на 4 для stosl
    
    cld                     ; Направление вперёд
    rep stosd               ; Заполняем память
    
    mov eax, [esp + 4]      ; Возвращаем исходный dst
    pop edi                 ; Восстанавливаем EDI
    ret

.label2_unaligned:
    xor edx, edx
    sub edx, edi
    and edx, 15             ; EDX = сколько байт нужно записать до выравнивания по 16 байт
    mov [edi], eax          ; Записываем первые 16 байт «вслепую»
    mov [edi + 4], eax
    mov [edi + 8], eax
    mov [edi + 12], eax
    sub ecx, edx            ; Уменьшаем счетчик n на количество выравнивающих байт (теперь тут валидный n)
    add edi, edx            ; Сдвигаем указатель EDI вперед до выровненной границы
    jmp .label1_loop        ; Переходим к основному циклу rep stosl
