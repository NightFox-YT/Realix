; Realix Print
; (C) v0.02 | 19.07.2025
; ============

; [Print] Вывод строки
; Параметры:
;  > ds:si - строка
print:
    ; Сохранение регистров
	push si
	push ax
	push bx

	cld          ; Сброс флага направления (df=0)
    mov ah, 0x0E ; 'TTY mode' (Вывод с прокруткой курсора)
	mov bh, 0    ; Номер страницы

.next_char:
	lodsb       ; Загрузка символа из si в al
	test al, al ; Проверка на 0 (Конец строки), логическое 'И'
	jz .done    ; Если 0 -> Переходим к .done
	
	int 0x10       ; Вызов BIOS
	jmp .next_char ; Символ не нулевой, -> возращаемся к .loop

.done:
    ; Восстановление регистров
	pop bx
	pop ax
	pop si
	ret ; Выход из функции