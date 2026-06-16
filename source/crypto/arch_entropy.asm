; ============================================================
; Copyright Gleb Obitotsky <https://github.com/oxxx1mif> 2026.
;
; License: GNU General Public License v3
; You can find the license file in the project root.
;
; Implementation version 0.1
; The code was written for Realix.
; ============================================================

; ============================================================
; ../source/crypto/arch_entropy.asm
;
; Предоставляет низкоуровневый, безопасный доступ к аппаратным
; источникам энтропии процессора: RDRAND и RDSEED.
;
; Поддержка i386 (cdecl) и x86-64 (System V ABI)
; ============================================================

global arch_has_rdrand
global arch_has_rdseed
global arch_get_rdrand32
global arch_get_rdrand64
global arch_get_rdseed32
global arch_get_rdseed64
global arch_get_entropy

%define ARCH_MAX_ENTROPY_WORDS  4096

section .text

; ============================================================
; int arch_has_rdrand(void)
;
; Возвращает:
;   1 — RDRAND поддерживается процессором
;   0 — не поддерживается
;
; Проверяет бит CPUID.01H:ECX[30]
; ============================================================
arch_has_rdrand:
    push ebx
    push ecx
    push edx

    xor eax, eax
    cpuid
    cmp eax, 1
    jb .no

    mov eax, 1
    cpuid
    xor eax, eax
    bt ecx, 30
    setc al                     ; очистка EAX

    pop edx
    pop ecx
    pop ebx
    ret
.no:
    xor eax, eax
    pop edx
    pop ecx
    pop ebx
    ret

; ============================================================
; int arch_has_rdseed(void)
;
; Возвращает:
;   1 — RDSEED поддерживается
;   0 — не поддерживается
;
; Проверяет бит CPUID.07H:EBX[18]
; ============================================================
arch_has_rdseed:
    push ebx
    push ecx
    push edx

    xor eax, eax
    cpuid
    cmp eax, 7
    jb .no

    mov eax, 7
    xor ecx, ecx
    cpuid
    xor eax, eax
    bt ebx, 18
    setc al

    pop edx
    pop ecx
    pop ebx
    ret
.no:
    xor eax, eax
    pop edx
    pop ecx
    pop ebx
    ret

; ============================================================
; int arch_get_rdrand32(uint32_t *out, uint32_t retries)
;
; Параметры:
;   out     — указатель куда записать 32-битное случайное число
;   retries — максимальное количество попыток (рекомендуется 10-32)
;
; Возвращает:
;   1 — успех, значение записано
;   0 — не удалось после retries попыток
; ============================================================
arch_get_rdrand32:
%if __BITS__ == 64
    test rdi, rdi
    jz .fail
    test esi, esi
    jz .fail

    mov ecx, esi
.retry:
    rdrand eax
    jc .ok
    pause
    dec ecx
    jnz .retry
.fail:
    xor eax, eax
    ret
.ok:
    mov [rdi], eax
    mov eax, 1
    ret
%else
    push ebp
    mov ebp, esp
    push ebx

    mov ebx, [ebp+8]        ; out
    mov ecx, [ebp+12]       ; retries
    test ebx, ebx
    jz .fail32

.retry32:
    rdrand eax
    jc .ok32
    pause
    dec ecx
    jnz .retry32

.fail32:
    xor eax, eax
    jmp .exit32
.ok32:
    mov [ebx], eax
    mov eax, 1
.exit32:
    pop ebx
    mov esp, ebp
    pop ebp
    ret
%endif

; ============================================================
; int arch_get_rdrand64(uint64_t *out, uint32_t retries)
;
; Аналогично arch_get_rdrand32, но для 64-битного значения
; ============================================================
arch_get_rdrand64:
%if __BITS__ == 64
    test rdi, rdi
    jz .fail
    test esi, esi
    jz .fail

    mov ecx, esi
.retry:
    rdrand rax
    jc .ok
    pause
    dec ecx
    jnz .retry
.fail:
    xor eax, eax
    ret
.ok:
    mov [rdi], rax
    mov eax, 1
    ret
%else
    xor eax, eax
    ret
%endif

; ============================================================
; int arch_get_rdseed32(uint32_t *out, uint32_t retries)
; ============================================================
arch_get_rdseed32:
%if __BITS__ == 64
    test rdi, rdi
    jz .fail
    test esi, esi
    jz .fail

    mov ecx, esi
.retry:
    rdseed eax
    jc .ok
    pause
    dec ecx
    jnz .retry
.fail:
    xor eax, eax
    ret
.ok:
    mov [rdi], eax
    mov eax, 1
    ret
%else
    push ebp
    mov ebp, esp
    push ebx

    mov ebx, [ebp+8]
    mov ecx, [ebp+12]
    test ebx, ebx
    jz .fail32

.retry32:
    rdseed eax
    jc .ok32
    pause
    dec ecx
    jnz .retry32

.fail32:
    xor eax, eax
    jmp .exit32
.ok32:
    mov [ebx], eax
    mov eax, 1
.exit32:
    pop ebx
    mov esp, ebp
    pop ebp
    ret
%endif

; ============================================================
; int arch_get_rdseed64(uint64_t *out, uint32_t retries)
;
; В 32-битном режиме возвращает 64-битное значение через EDX:EAX
; ============================================================
arch_get_rdseed64:
%if __BITS__ == 64
    test rdi, rdi
    jz .fail
    test esi, esi
    jz .fail

    mov ecx, esi
.retry:
    rdseed rax
    jc .ok
    pause
    dec ecx
    jnz .retry
.fail:
    xor eax, eax
    ret
.ok:
    mov [rdi], rax
    mov eax, 1
    ret
%else
    ; 32-bit: возвращаем 64-бит через EDX:EAX
    push ebp
    mov ebp, esp
    push ebx

    mov ebx, [ebp+8]        ; out
    mov ecx, [ebp+12]       ; retries
    test ebx, ebx
    jz .fail32

.retry32:
    rdseed eax
    jnc .next
    pause
    dec ecx
    jnz .retry32
    jmp .fail32

.next:
    push eax                ; сохраняем первое слово
    rdseed eax
    jnc .fail_pop
    mov edx, eax            ; второе слово в EDX
    pop eax                 ; первое слово обратно в EAX
    mov [ebx], eax
    mov [ebx+4], edx
    mov eax, 1
    jmp .exit32

.fail_pop:
    pop eax
.fail32:
    xor eax, eax
.exit32:
    pop ebx
    mov esp, ebp
    pop ebp
    ret
%endif

; ============================================================
; int arch_get_entropy(void *buffer, uint32_t words,
;                      uint32_t retries, uint32_t source)
;
; Параметры:
;   buffer  — указатель на память для записи
;   words   — сколько 32-битных слов нужно сгенерировать
;   retries — максимум попыток на одно слово
;   source  — 1 = RDRAND, 2 = RDSEED
;
; Возвращает:
;   1 — успешно заполнено words слов
;   0 — ошибка
;
; Рекомендации по использованию:
;   - Для начального seeding лучше использовать source=2 (RDSEED)
;   - Для быстрой подкачки энтропии — source=1 (RDRAND)
; ============================================================
arch_get_entropy:
%if __BITS__ == 64
    ; SysV ABI: rdi, rsi, rdx, rcx
    test rdi, rdi
    jz .fail
    test rsi, rsi
    jz .fail
    test rdx, rdx
    jz .fail
    cmp rsi, ARCH_MAX_ENTROPY_WORDS
    ja .fail

    push rbx
    push r12
    push r13
    push r14
    push r15

    mov r12, rdi        ; buffer
    mov r13, rsi        ; words
    mov r14, rdx        ; retries
    mov r15, rcx        ; source (1=RDRAND, 2=RDSEED)

.loop:
    test r13, r13
    jz .success

    mov rdi, r12
    mov esi, r14d

    cmp r15d, 2
    je .do_rdseed

    call arch_get_rdrand32
    jmp .check
.do_rdseed:
    call arch_get_rdseed32
.check:
    test eax, eax
    jz .fail_restore

    add r12, 4
    dec r13
    jmp .loop

.success:
    mov eax, 1
.fail_restore:
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    ret
.fail:
    xor eax, eax
    ret

%else
    ; i386 cdecl
    push ebp
    mov ebp, esp
    push ebx
    push esi
    push edi

    mov edi, [ebp+8]     ; buffer
    mov ebx, [ebp+12]    ; words
    mov esi, [ebp+16]    ; retries
    mov edx, [ebp+20]    ; source

    test edi, edi
    jz .fail32
    test ebx, ebx
    jz .fail32
    test esi, esi
    jz .fail32
    cmp ebx, ARCH_MAX_ENTROPY_WORDS
    ja .fail32

.loop32:
    test ebx, ebx
    jz .success32

    push edx
    push esi             ; retries
    push edi             ; buffer

    cmp edx, 2
    je .rdseed32
    call arch_get_rdrand32
    jmp .cleanup
.rdseed32:
    call arch_get_rdseed32
.cleanup:
    add esp, 12

    test eax, eax
    jz .fail32

    add edi, 4
    dec ebx
    jmp .loop32

.success32:
    mov eax, 1
    jmp .exit32
.fail32:
    xor eax, eax
.exit32:
    pop edi
    pop esi
    pop ebx
    mov esp, ebp
    pop ebp
    ret
%endif